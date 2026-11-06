use crate::physics::Ship;
use crate::track::BakedTrack;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::net::UdpSocket;

pub const DEFAULT_UDP_PORT: u16 = 47999;
pub const BROADCAST_ADDR: &str = "255.255.255.255:47999";
pub const NET_SEND_INTERVAL_SEC: f32 = 1.0 / 30.0;
pub const UDP_RECV_BUFFER_SIZE: usize = 512;
pub const DEAD_RECKONING_BLEND: f32 = 0.35;
pub const REMOTE_BOOST_PULSE_SEC: f32 = 0.25;
pub const MAX_NET_PEERS: usize = 4;

/// Network packet for UDP broadcast ship state synchronization.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NetShipPacket {
    pub room_code: [u8; 4],
    pub player_id: u8,
    pub segment_id: u8,
    pub s: f32,
    pub r: f32,
    pub h: f32,
    pub v_s: f32,
    pub v_r: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub boosting: bool,
}

impl NetShipPacket {
    pub fn from_ship(ship: &Ship, room_code: [u8; 4]) -> Self {
        Self {
            room_code,
            player_id: ship.id,
            segment_id: ship.segment_id as u8,
            s: ship.s,
            r: ship.r,
            h: ship.h,
            v_s: ship.v_s,
            v_r: ship.v_r,
            yaw: ship.yaw_offset,
            pitch: ship.pitch_offset,
            roll: ship.visual_roll,
            boosting: ship.boost_timer > 0.0,
        }
    }

    pub fn has_finite_state(&self) -> bool {
        self.s.is_finite()
            && self.r.is_finite()
            && self.h.is_finite()
            && self.v_s.is_finite()
            && self.v_r.is_finite()
            && self.yaw.is_finite()
            && self.pitch.is_finite()
            && self.roll.is_finite()
    }

    pub fn is_valid_for_session(&self, expected_room: [u8; 4], segment_count: usize) -> bool {
        self.room_code == expected_room
            && self.player_id > 0
            && (self.segment_id as usize) < segment_count
            && self.has_finite_state()
    }
}

pub fn encode_room_code(code: &str) -> [u8; 4] {
    let mut out = [b'A', b'G', b'Y', b'4'];
    for (i, b) in code.bytes().take(4).enumerate() {
        out[i] = b;
    }
    out
}

pub fn apply_packet_to_remote_ship(remote: &mut Ship, pkt: &NetShipPacket) {
    remote.segment_id = pkt.segment_id as usize;
    remote.s += (pkt.s - remote.s) * DEAD_RECKONING_BLEND;
    remote.r += (pkt.r - remote.r) * DEAD_RECKONING_BLEND;
    remote.h = pkt.h;
    remote.v_s = pkt.v_s;
    remote.v_r = pkt.v_r;
    remote.yaw_offset = pkt.yaw;
    remote.pitch_offset = pkt.pitch;
    remote.visual_roll = pkt.roll;
    if pkt.boosting {
        remote.boost_timer = REMOTE_BOOST_PULSE_SEC;
    }
}

/// Network session broadcasting and receiving ship state over local UDP LAN.
pub struct NetSession {
    pub room_code: String,
    pub is_online: bool,
    pub peer_count: usize,
    pub seen_peers: [bool; MAX_NET_PEERS],
    socket: Option<UdpSocket>,
    peer_addr: String,
    send_accum: f32,
}

impl NetSession {
    pub fn new(room_code: &str) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:0").ok();
        if let Some(ref s) = socket {
            let _ = s.set_nonblocking(true);
            let _ = s.set_broadcast(true);
        }

        Self {
            room_code: room_code.to_string(),
            is_online: false,
            peer_count: 0,
            seen_peers: [false; MAX_NET_PEERS],
            socket,
            peer_addr: BROADCAST_ADDR.to_string(),
            send_accum: 0.0,
        }
    }

    pub fn toggle_online(&mut self) {
        self.is_online = !self.is_online;
        if self.is_online {
            let bind_target = format!("0.0.0.0:{}", DEFAULT_UDP_PORT);
            if let Ok(listener) =
                UdpSocket::bind(&bind_target).or_else(|_| UdpSocket::bind("0.0.0.0:0"))
            {
                let _ = listener.set_nonblocking(true);
                let _ = listener.set_broadcast(true);
                self.socket = Some(listener);
            }
        } else {
            self.socket = None;
            self.seen_peers = [false; MAX_NET_PEERS];
            self.peer_count = 0;
        }
    }

    /// Broadcast local player ship state and apply received updates to remote peers.
    pub fn tick_sync(&mut self, ships: &mut [Ship], track: &BakedTrack, dt: f32) {
        if !self.is_online {
            return;
        }

        self.send_accum += dt;
        if self.send_accum >= NET_SEND_INTERVAL_SEC {
            self.send_accum = 0.0;
            if let (Some(ref sock), Some(player)) = (&self.socket, ships.first()) {
                let code = encode_room_code(&self.room_code);
                let pkt = NetShipPacket::from_ship(player, code);
                if let Ok(bytes) = serde_json::to_vec(&pkt) {
                    let _ = sock.send_to(&bytes, &self.peer_addr);
                }
            }
        }

        let Some(ref sock) = self.socket else {
            return;
        };
        let expected_room = encode_room_code(&self.room_code);
        let mut buf = [0u8; UDP_RECV_BUFFER_SIZE];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((len, _src)) => {
                    if let Ok(pkt) = serde_json::from_slice::<NetShipPacket>(&buf[..len]) {
                        if pkt.is_valid_for_session(expected_room, track.segments.len()) {
                            if (pkt.player_id as usize) < self.seen_peers.len() {
                                self.seen_peers[pkt.player_id as usize] = true;
                            }
                            self.peer_count =
                                self.seen_peers.iter().filter(|&&seen| seen).count();

                            if let Some(remote) =
                                ships.iter_mut().find(|s| s.id == pkt.player_id)
                            {
                                apply_packet_to_remote_ship(remote, &pkt);
                                remote.segment_id = remote
                                    .segment_id
                                    .min(track.segments.len().saturating_sub(1));
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_room_code_pads_and_truncates() {
        assert_eq!(encode_room_code("AGY4"), [b'A', b'G', b'Y', b'4']);
        assert_eq!(encode_room_code("AB"), [b'A', b'B', b'Y', b'4']);
        assert_eq!(encode_room_code("LONGERCODE"), [b'L', b'O', b'N', b'G']);
        assert_eq!(encode_room_code(""), [b'A', b'G', b'Y', b'4']);
    }

    #[test]
    fn test_packet_validation_rejects_wrong_room_and_nan_floats() {
        let valid_room = [b'R', b'A', b'C', b'E'];
        let base_pkt = NetShipPacket {
            room_code: valid_room,
            player_id: 1,
            segment_id: 0,
            s: 10.0,
            r: 0.0,
            h: 1.25,
            v_s: 50.0,
            v_r: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            boosting: false,
        };

        assert!(base_pkt.has_finite_state());
        assert!(base_pkt.is_valid_for_session(valid_room, 4));

        assert!(!base_pkt.is_valid_for_session([b'D', b'I', b'F', b'F'], 4));

        let mut pkt = base_pkt;
        pkt.player_id = 0;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.segment_id = 4;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.s = f32::NAN;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.r = f32::NAN;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.yaw = f32::INFINITY;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.pitch = f32::NEG_INFINITY;
        assert!(!pkt.is_valid_for_session(valid_room, 4));

        let mut pkt = base_pkt;
        pkt.roll = f32::NAN;
        assert!(!pkt.is_valid_for_session(valid_room, 4));
    }
}
