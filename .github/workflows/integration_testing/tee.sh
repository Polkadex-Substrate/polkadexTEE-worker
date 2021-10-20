#!/usr/bin/expect

set timeout -1
spawn polkadexTEE-worker/docker-start.sh

expect "root@"
send -- "cd work\rcd polkadexTEE-worker\r./ci/install_rust.sh\rmake\rexit\r"

expect eof