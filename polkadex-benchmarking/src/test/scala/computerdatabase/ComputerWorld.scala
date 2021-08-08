package computerdatabase

import scala.concurrent.duration._

import java.util.concurrent.ThreadLocalRandom

import io.gatling.core.Predef._
import io.gatling.http.Predef._

class PolkadexBenchmarking extends Simulation {

  val httpProtocol = http.wsBaseUrl("ws://IP:8020")
  val message = """{"jsonrpc":"2.0","method":"place_order","params":[180,88,204,28,50,1,195,127,37,170,225,17,82,25,135,210,190,231,76,144,3,80,252,2,85,218,240,146,61,81,194,95,213,2,1,4,212,53,147,199,21,253,211,28,97,20,26,189,4,169,159,214,130,44,133,88,133,76,205,227,154,86,132,231,165,109,162,125,212,53,147,199,21,253,211,28,97,20,26,189,4,169,159,214,130,44,133,88,133,76,205,227,154,86,132,231,165,109,162,125,1,0,28,116,114,117,115,116,101,100,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,10,176,61,15,174,12,109,2,183,164,220,208,129,9,236,190,104,169,7,218,32,209,86,70,246,185,130,237,22,125,176,4,5,107,199,66,123,148,126,22,150,11,216,99,141,102,204,189,13,121,62,56,60,71,116,2,108,103,48,217,61,95,142,129],"id":1}"""
  val jsonFileFeeder = jsonFile("data.json")

  val scene = scenario("testWebSocket").feed(jsonFileFeeder)
    .exec(ws("openSocket").connect("")
      .onConnected(exec(ws("sendMessage").sendText(StringBody("""{"jsonrpc": "${jsonrpc}", "method": "${method}", "params": ${params.jsonStringify()}, "id": ${id}}"""))
        .await(20)(ws.checkTextMessage("check1").check(regex(".*4,0,0,0.*")
          .saveAs("myMessage"))))))
    .exec(session => session {
      println(session("myMessage").as[String])
      session
    })
    .exec(ws("closeConnection").close)

  setUp(scene.inject(atOnceUsers(1000)).protocols(httpProtocol))
}
