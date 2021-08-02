package computerdatabase

import scala.concurrent.duration._

import java.util.concurrent.ThreadLocalRandom

import io.gatling.core.Predef._
import io.gatling.http.Predef._

class PolkadexBenchmarking extends Simulation {

  val httpProtocol = http.wsBaseUrl("ws://88.198.24.21:8020")

  val message = """{"jsonrpc": "2.0","method": "place_order","params": [65, 61, 211, 138, 32, 161, 21, 135, 119, 9, 73, 45, 139, 62, 46, 163, 53, 200, 61, 38, 155, 91, 91, 249, 210, 145, 60, 178, 179, 171, 71, 193, 213, 2, 1, 4, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125, 212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125, 1, 0, 28, 116, 114, 117, 115, 116, 101, 100, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 122, 4, 79, 138, 32, 252, 120, 79, 100, 117, 247, 183, 205, 3, 67, 88, 172, 14, 16, 29, 81, 162, 198, 134, 99, 201, 64, 176, 7, 172, 38, 104, 106, 129, 107, 99, 253, 123, 151, 11, 5, 166, 131, 118, 87, 9, 212, 200, 61, 19, 190, 125, 143, 81, 72, 215, 99, 242, 189, 122, 245, 131, 34, 140],"id": 1}"""
  val jsonFileFeeder = jsonFile("data.json")

/*val scene = scenario("testWebSocket").feed(jsonFileFeeder)
    .exec(ws("openSocket").connect("")
      .onConnected(exec(ws("sendMessage").sendText(StringBody("""{"jsonrpc": "${jsonrpc}", "method": "${method}", "params": ${params}.toArray, "id": ${id}}"""))
        .await(20)(ws.checkTextMessage("check1").check(regex(".*")
          .saveAs("myMessage"))))))*/

  val scene = scenario("testWebSocket")
    .exec(ws("openSocket").connect("")
      .onConnected(repeat(100){exec(ws("sendMessage").sendText(message)
        .await(20)(ws.checkTextMessage("check1").check(regex(".*4,0,0,0.*")
          .saveAs("myMessage"))))}))
    // created custom checks for checking my response

    .exec(session => session{
      println(session("myMessage").as[String])
      session
    })
    //created the session for printing the response and type-casted it to String

    .exec(ws("closeConnection").close)
  //terminating the current websocket connection

  setUp(scene.inject(atOnceUsers(500)).protocols(httpProtocol))

}
