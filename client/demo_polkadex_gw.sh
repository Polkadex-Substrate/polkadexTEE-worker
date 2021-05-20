#!/bin/bash

# setup:
# run all on localhost:
#   substratee-node purge-chain --dev
#   substratee-node --tmp --dev -lruntime=debug
#   rm chain_relay_db.bin
#   substratee-worker init_shard
#   substratee-worker shielding-key
#   substratee-worker signing-key
#   substratee-worker run
#
# then run this script

# usage:
#  demo_direct_call.sh <NODEPORT> <WORKERRPCPORT>

# using default port if none given as arguments
NPORT=${1:-9944}
RPORT=${2:-2000}

echo "Using node-port ${NPORT}"
echo "Using worker-rpc-port ${RPORT}"
echo ""

CLIENT="../bin/substratee-client -p ${NPORT} -P ${RPORT}"

echo "* Query on-chain enclave registry:"
${CLIENT} list-workers
echo ""

# does this work when multiple workers are in the registry?
read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE:[[:space:]]/ { print $2 }')

# example for calling 'place_order'
echo "Alice places order with proxy account"
$CLIENT trusted place_order --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
 --marketid=btcusd --markettype=trusted --ordertype=limit --orderside=ask --quantity=987345 \
 --mrenclave $MRENCLAVE --direct

echo "Get balance of Alice"
$CLIENT trusted get_balance --accountid=//AliceIncognito --tokenid=btc --mrenclave $MRENCLAVE

echo "Cancel order"
$CLIENT trusted cancel_order --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
 --marketid=btcusd --markettype=trusted --ordertype=limit --orderside=ask --quantity=987345 \
 --mrenclave $MRENCLAVE --direct

echo "Withdraw"
$CLIENT trusted withdraw --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
 --tokenid=btc --quantity=293 \
 --mrenclave $MRENCLAVE