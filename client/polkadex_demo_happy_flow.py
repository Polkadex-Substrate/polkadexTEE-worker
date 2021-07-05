#!/usr/bin/env python3

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


"""
demo script running happy flow openfinex commands
"""

import optparse

from polkadex_commands import *
#from polkadex_commands import direct_get_balance, direct_place_order, direct_withdraw, withdraw, token_balance
#from polkadex_commands import write_cli, read_mrenclave, register_account, register_proxy, deposit, await_block

alice = '//Alice'
bob = '//Bob'
aliceIco = '//AliceIcognito'
bobIco = '//BobIcognito'
tokenA = 'btc'
tokenB = 'usd'

if __name__ == '__main__':
    parser = optparse.OptionParser()
    parser.add_option('-p', '--node-port', dest='node', help='Node port', type=int)
    parser.add_option('-P', '--worker-port', dest='worker', help='Worker port', type=int)
    parser.add_option('-t', '--test-run', dest='test', help='indicates at which stage the test is currtly at', type=int)
    (options, args) = parser.parse_args()
    write_cli(options)
    read_mrenclave()


    # happy flow:
    #1: Alice and Bob both create an account and receive funds from faucet (some native tokens as well
    # as 200 tokenA for Alice and 200 tokenB for Bob)
    register_account(alice)
    register_account(bob)

    #2 Alice and Bob both create and register a proxy account
    register_proxy(alice, aliceIco)
    register_proxy(bob, bobIco)


    print("Checking balance of Alice onchain:")
    token_balance(alice, tokenA)
    token_balance(alice, tokenB)

    print("Checking balance of Bob onchain:")
    token_balance(bob, tokenA)
    token_balance(bob, tokenB)

    print("Start depositing funds from onchain to offchain:")
    #3 Alice deposits 0.5 tokenA
    deposit(alice, 500_000_000_000_000_000, tokenA)

    #4 Bob deposits 0.5 tokenB
    deposit(bob, 500_000_000_000_000_000, tokenB)

    print("Check new onchain balances:")
    token_balance(alice, tokenA)
    token_balance(bob, tokenB)

    await_block() # wait some time to ensure enclave has read new block from main chain
    print("And offchain balances accordingly:")
    direct_get_balance(alice, tokenA)
    direct_get_balance(bob, tokenB)

    #5 Alice places a limit order selling 50 tokenA at a limit of 40 tokenB
    print("Alice places a sell order 0.05 btc for 0.05 usd")
    direct_place_order(alice, None, tokenA, tokenB, 'ask', 50_000_000_000_000_000, 'limit', 1_000_000_000_000_000_000)
    await_block()

    #6 Bob places a limit order buying 50 tokenA at a limit of 60 tokenB
    print("Bob places a buy order 0.05 usd for 0.05 btc")
    direct_place_order(bob, None, tokenA, tokenB, 'bid', 50_000_000_000_000_000, 'limit', 1_000_000_000_000_000_000)
    await_block()

    #7 The matching engine clears the match, sends it to the gateway
    #8 The gateway settles the match, publishes all details

    await_block() # wait some time to matching engine had some time
    #9 The offchain balance of Alice is 50 tokenA plus 50 tokenB
    print("Checking if transfer was successful:")
    direct_get_balance(alice, tokenA)
    direct_get_balance(alice, tokenB)
    #10 The offchain balance of Bob is 50 tokenA plus 50 tokenB
    direct_get_balance(bob, tokenA)
    direct_get_balance(bob, tokenB)

    #11 Alice withdraws all her tokenB through direct call to gateway
    print("Alice and Bob withdraw their newly traded tokens:")
    direct_withdraw(alice, None, tokenB, 50_000_000_000_000_000)

    #12 Bob withdraws all his tokenA through indirect extrinsic
    withdraw(bob, tokenA, 50_000_000_000_000_000)

    #13 The offchain balance of Alice is zero tokenB and Bob is zero tokenA
    print("Chech if transfer from offchain to onchain was successful:")
    direct_get_balance(alice, tokenB)
    direct_get_balance(bob, tokenA)

    #14 The onchain balance of Alice is 50 tokenB and Bob is 50 tokenA
    token_balance(alice, tokenB)
    token_balance(bob, tokenA)