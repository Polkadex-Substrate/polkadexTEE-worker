#!/usr/bin/expect

set timeout -1
spawn ./.github/workflows/integration_testing/openware-exec.sh

expect "mysql>"
send -- "INSERT INTO members(uid,email,level,role,state,created_at,updated_at,username) VALUES('ALICE','protected',3,'member','active','2021-05-20 14:43:58','2021-05-20 14:41:41', '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');\n"

expect "mysql>"
send -- "INSERT INTO members(uid,email,level,role,state,created_at,updated_at,username) VALUES('BOB','protected_bob',3,'member','active','2021-05-20 14:43:58','2021-05-20 14:41:41', '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty');\n"

expect "mysql>"
send -- "quit\n"

expect eof