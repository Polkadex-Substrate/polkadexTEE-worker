#!/bin/bash

#  This file is part of Polkadex.
#  Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
#  SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#  This program is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
#  GNU General Public License for more details.
#  You should have received a copy of the GNU General Public License
#  along with this program. If not, see <https://www.gnu.org/licenses/>.

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
 --marketbase=polkadex --marketquote=dot --markettype=trusted --ordertype=limit --orderside=ask --quantity=987345 \
 --mrenclave $MRENCLAVE --direct

echo "Get balance of Alice"
$CLIENT trusted get_balance --accountid=//AliceIncognito \
 --tokenid=dot \
 --mrenclave $MRENCLAVE --direct

echo "Cancel order"
$CLIENT trusted cancel_order --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
 --orderid=oijef03jaf \
 --mrenclave $MRENCLAVE --direct

echo "Withdraw"
$CLIENT trusted withdraw --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
 --tokenid=dot --quantity=293 \
 --mrenclave $MRENCLAVE --direct