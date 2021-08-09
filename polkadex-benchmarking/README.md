Polkadex Benchmarking
=========================
## Prerequisite

### Install SDKMAN
```bash
$ curl -s "https://get.sdkman.io" | bash
$ source "$HOME/.sdkman/bin/sdkman-init.sh"
```
### Install Java and Scala
```bash
$ sdk install java $(sdk list java | grep -o "8\.[0-9]*\.[0-9]*\.hs-adpt" | head -1)
$ sdk install sbt
```
### Create Payload file
```bash
$ git clone https://github.com/Polkadex-Substrate/payload-creator-polkadextee
$ cargo run --feature encode MRENCLAVE-ID
```
Now copy content from ```Payload.txt``` and paste in inside ```/polkadex-benchmarking/src/test/resources/data.json```.

## Start Benchmark
### Start SBT

```bash
$ sbt
```

###Run all simulations

```bash
> Gatling / test
```
You can also modify User strength from here:-
```scala
  setUp(scene.inject(atOnceUsers(TOTAL_USERS)).protocols(httpProtocol))
```
