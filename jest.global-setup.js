import { server, ready } from "./tests/proxy-server/index.mjs"

export default async function () {
  await ready()
  server.listen(8080)
}
