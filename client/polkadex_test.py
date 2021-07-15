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

    print("Start depositing funds from onchain to offchain:")
    #3 Alice deposits 0.5 tokenA
    deposit(alice, 500_000_000_000_000_000, tokenA)

    await_block() # wait some time to ensure enclave has read new block from main chain
    print("And offchain balances accordingly:")
    direct_get_balance(alice, tokenA)

    #11 Alice withdraws all her tokenB through direct call to gateway
    print("Alice and Bob withdraw their newly traded tokens:")
    withdraw(alice, tokenA, 50_000_000_000_000_000)