/**
 * AGY-RACER Cloudflare Worker
 * - 4-Letter Room Code WebRTC Signaling (POST /room/:code, GET /room/:code)
 * - Public Matchmaking Lobby Directory (GET /lobbies)
 * - Async Ghost Replay & Time-Trial Leaderboards (GET /ghosts/:track, POST /ghosts/:track)
 */

export const DEFAULT_ROOM_CODE = "AGY4";
export const ROOM_TTL_SEC = 300;
export const MAX_PUBLIC_LOBBIES = 20;
export const DEFAULT_TRACK_SLUG = "circuit_01";
export const MIN_VALID_LAP_SEC = 5.0;
export const MIN_GHOST_SAMPLES = 10;
export const MAX_GHOST_SAMPLES = 3600;

export function isValidGhostPayload(ghost) {
  return Boolean(
    ghost &&
      typeof ghost.lap_time_sec === "number" &&
      Number.isFinite(ghost.lap_time_sec) &&
      ghost.lap_time_sec > MIN_VALID_LAP_SEC &&
      Array.isArray(ghost.frames) &&
      ghost.frames.length >= MIN_GHOST_SAMPLES &&
      ghost.frames.length <= MAX_GHOST_SAMPLES
  );
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    };

    if (request.method === "OPTIONS") {
      return new Response(null, { headers: corsHeaders });
    }

    if (url.pathname === "/lobbies" && request.method === "GET") {
      const list = (await env.AGY_KV?.get("public_lobbies", "json")) || [];
      return Response.json(list, { headers: corsHeaders });
    }

    if (url.pathname.startsWith("/room/")) {
      const code = url.pathname.split("/")[2]?.toUpperCase() || DEFAULT_ROOM_CODE;
      const key = `room_${code}`;
      if (request.method === "POST") {
        let body;
        try {
          body = await request.json();
        } catch {
          return Response.json(
            { error: "Invalid JSON body" },
            { status: 400, headers: corsHeaders }
          );
        }

        await env.AGY_KV?.put(key, JSON.stringify(body), { expirationTtl: ROOM_TTL_SEC });

        if (body && body.public === true) {
          const lobbies = (await env.AGY_KV?.get("public_lobbies", "json")) || [];
          const updated = lobbies.filter((l) => l.code !== code);
          updated.push({ code, timestamp: Date.now() });
          await env.AGY_KV?.put(
            "public_lobbies",
            JSON.stringify(updated.slice(-MAX_PUBLIC_LOBBIES)),
            { expirationTtl: ROOM_TTL_SEC }
          );
        }

        return Response.json({ ok: true, code }, { headers: corsHeaders });
      } else {
        const room = (await env.AGY_KV?.get(key, "json")) || null;
        return Response.json(room, { headers: corsHeaders });
      }
    }

    if (url.pathname.startsWith("/ghosts/")) {
      const track = url.pathname.split("/")[2] || DEFAULT_TRACK_SLUG;
      const key = `ghost_${track}`;
      if (request.method === "POST") {
        let ghost;
        try {
          ghost = await request.json();
        } catch {
          return Response.json(
            { error: "Invalid JSON body" },
            { status: 400, headers: corsHeaders }
          );
        }

        if (!isValidGhostPayload(ghost)) {
          return Response.json(
            { error: "Invalid ghost data" },
            { status: 400, headers: corsHeaders }
          );
        }

        const existing = await env.AGY_KV?.get(key, "json");
        if (
          existing &&
          typeof existing.lap_time_sec === "number" &&
          ghost.lap_time_sec >= existing.lap_time_sec
        ) {
          return Response.json(
            { saved: false, reason: "Slower than or equal to existing record" },
            { status: 200, headers: corsHeaders }
          );
        }

        await env.AGY_KV?.put(key, JSON.stringify(ghost));
        return Response.json(
          { saved: true, lap_time_sec: ghost.lap_time_sec },
          { headers: corsHeaders }
        );
      } else {
        const ghost = (await env.AGY_KV?.get(key, "json")) || null;
        return Response.json(ghost, { headers: corsHeaders });
      }
    }

    return Response.json(
      { service: "agy-racer-worker", status: "online" },
      { headers: corsHeaders }
    );
  },
};
