const WASM_PATH = "/diff_bg.wasm";
const WASM_BR_PATH = "/diff_bg.wasm.br";
// Fixed filename, so keep the cache window short enough to pick up deploys.
const WASM_CACHE_CONTROL = "public, max-age=0, must-revalidate";

function acceptsBrotli(request) {
  const header = request.headers.get("Accept-Encoding");
  if (!header) return false;

  return header.split(",").some((entry) => {
    const [coding, ...params] = entry.trim().toLowerCase().split(";");
    if (coding !== "br") return false;

    const q = params.map((param) => param.trim()).find((param) => param.startsWith("q="));
    return q === undefined || Number.parseFloat(q.slice(2)) > 0;
  });
}

function brotliHeaders(etag) {
  const headers = new Headers({
    "Content-Type": "application/wasm",
    "Content-Encoding": "br",
    "Vary": "Accept-Encoding",
    "Cache-Control": WASM_CACHE_CONTROL,
  });
  if (etag) headers.set("ETag", etag);
  return headers;
}

async function serveBrotliWasm(request, env, url) {
  // Ask the asset layer for the stored bytes. Without this, an intermediary can
  // transparently re-compress the `.br` file, which would double-encode it.
  const headers = new Headers({ "Accept-Encoding": "identity" });
  const ifNoneMatch = request.headers.get("If-None-Match");
  if (ifNoneMatch) headers.set("If-None-Match", ifNoneMatch);

  const asset = await env.ASSETS.fetch(
    new Request(new URL(WASM_BR_PATH, url), { headers }),
  );

  if (asset.status === 304) {
    return new Response(null, {
      status: 304,
      headers: brotliHeaders(asset.headers.get("ETag")),
    });
  }
  if (!asset.ok) return null;

  const encoding = asset.headers.get("Content-Encoding");
  if (encoding && encoding.toLowerCase() !== "identity") return null;

  return new Response(asset.body, {
    headers: brotliHeaders(asset.headers.get("ETag")),
  });
}

async function serveOriginalWasm(request, env) {
  const response = await env.ASSETS.fetch(request);
  if (response.status !== 200) return response;

  const headers = new Headers(response.headers);
  headers.set("Content-Type", "application/wasm");
  headers.set("Cache-Control", WASM_CACHE_CONTROL);
  headers.set("Vary", "Accept-Encoding");
  headers.delete("Content-Length");

  return new Response(response.body, { status: 200, headers });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (url.pathname === WASM_PATH) {
      if (request.method === "GET" && acceptsBrotli(request)) {
        try {
          const response = await serveBrotliWasm(request, env, url);
          if (response) return response;
        } catch {
          // Precompressed asset unavailable; serve the original wasm below.
        }
      }

      return serveOriginalWasm(request, env);
    }

    return new Response("404 - Not Found", { status: 404 });
  },
};
