Polkadex Benchmarking
=========================
## Install SDKMAN
```bash
$ curl -s "https://get.sdkman.io" | bash
$ source "$HOME/.sdkman/bin/sdkman-init.sh"
```
## Install Java and Scala
```bash
sdk install java $(sdk list java | grep -o "8\.[0-9]*\.[0-9]*\.hs-adpt" | head -1)
sdk install sbt
```

Start SBT
---------
```bash
$ sbt
```

Run all simulations
-------------------

```bash
> Gatling / test
```