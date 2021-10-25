#!/usr/bin/expect

set timeout -1
spawn ./docker-start.sh

expect "root@"
send -- "cd work\r./ci/install_rust.sh\rBENCHMARK=1 make\rexit\r"

expect eof