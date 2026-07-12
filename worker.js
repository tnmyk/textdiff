export default {
  async fetch(request, env) {
    return new Response("404 - Not Found", { status: 404 });
  }
};
