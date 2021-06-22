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

import subprocess
import sys
import optparse

#cli = ["./encointer-client", "-u", "wss://cantillon.encointer.org", "-p", "443", "-U", "wss://substratee03.scs.ch", "-P", "443"]
mrenclave_filename = "mrenclave.b58"
MRENCLAVE = ""
alice = '//Alice'
bob = '//Bob'
aliceIco = '//AliceIcognito'
bobIco = '//BobIcognito'
tokenA = 'btc'
tokenB = 'usd'
direct = '--direct'
markettype = ['--markettype=spot']
cli = ["../bin/substratee-client"]
PRECISION = 1_000_000_000_000_000_000

def direct_tail():
    return ["--mrenclave=" + MRENCLAVE, direct]
def acc_arg(acc):
    return ["--accountid=" + acc]
def proxy_arg(proxy):
    return ["--proxyaccountid=" + proxy]
def token_arg(token):
    return ["--tokenid=" + token]
def quantity_arg(quantity):
    return ["--quantity=" + str(quantity)]
def price_arg(price):
    return ["--price=" + str(price)]
def quote_arg(quote):
    return ["--marketquote=" + quote]
def base_arg(base):
    return ["--marketbase=" + base]
def orderside_arg(side):
    return ["--orderside=" + side]
def ordertype_arg(type):
    return ["--ordertype=" + type]
def orderid_arg(orderid):
    return ["--orderid=" + orderid]

def write_cli(options):
    global cli
    #default values
    if not (options.worker is None):
        cli.append('-P')
        cli.append(str(options.worker))
    if not (options.node is None):
        cli.append('-p')
        cli.append(str(options.node))

def read_mrenclave():
    """ reads the mrenclave from list-workers, and if not available from the file  /mrenclave.b58"""
    global MRENCLAVE
    ret = subprocess.run(cli + ["list-workers"], stdout=subprocess.PIPE)
    lines = ret.stdout.decode("utf-8").splitlines()
    workers = []
    if lines:
        for line in lines:
            if "MRENCLAVE" in line:
                mrenclave = line.split()
                workers.append(mrenclave[1].strip())
    else:
        # open file instead
        with open(mrenclave_filename) as f:
            workers = f.readlines()
    MRENCLAVE = workers[0]
    print("Using mrenclave of first worker as default: " + MRENCLAVE)
    return workers

def await_block():
    subprocess.run(cli + ["listen", "-b", "1"], stdout=subprocess.PIPE)

def register_account(acc):
    """ ./substratee-client -p 9994 -P 2094 register-account //Alice """
    print("Registering " + acc)
    ret = subprocess.run(cli + ["register-account"] + [acc], stdout=subprocess.PIPE)
    #print(ret.stdout.strip())
    await_block()
    return ret.stdout.decode("utf-8").strip()

def register_proxy(acc, proxy):
    """ ./substratee-client -p 9994 -P 2094 register-proxy //Alice //AliceIcognito"""
    print("Registering proxy account " + proxy + " for " + acc)
    ret = subprocess.run(cli + ["register-proxy"] + [acc] + [proxy], stdout=subprocess.PIPE)
    #print(ret.stdout.strip())
    await_block()
    return ret.stdout.decode("utf-8").strip()

def deposit(acc, quantity, token):
    """ ./substratee-client -p 9994 -P 2094 deposit --accountid=//Alice --tokenid=polkadex --quantity=10000 """
    print("Deposit " + str(quantity/PRECISION) + " " + token + " to " + acc)
    ret = subprocess.run(cli + ["deposit"] + acc_arg(acc) + quantity_arg(quantity) + token_arg(token), stdout=subprocess.PIPE)
    #print(ret.stdout.strip())
    await_block()
    return ret.stdout.decode("utf-8").strip()

def withdraw(acc, quantity, token):
    """  ./substratee-client -p 9994 -P 2094 withdraw --accountid=//Bob --tokenid=dot --quantity=1000 """
    print("Withdraw " + str(quantity) + " " + token + " from " + acc)
    ret = subprocess.run(cli + ["withdraw"] + acc_arg(acc) + quantity_arg(quantity) + token_arg(token), stdout=subprocess.PIPE)
    print(ret.stdout.strip())
    await_block()
    return ret.stdout.decode("utf-8").strip()

def direct_get_balance(acc, token):
    """ ./substratee-client -p 9994 -P 2094 trusted get_balance --accountid=//AliceIncognito --tokenid=dot \
    --mrenclave $MRENCLAVE --direct
    """
    ret = subprocess.run(cli + ["trusted", "get_balance"] + acc_arg(acc) + token_arg(token) + direct_tail(), stdout=subprocess.PIPE)
    print("Balance of " + acc + " " + str(ret.stdout))
    return ret.stdout.decode("utf-8").strip()

def direct_place_order(acc, proxy, base, quote, side, quantity, ordertype, price):
    """ ./substratee-client -p 9994 -P 2094 trusted place_order --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
    --marketbase=polkadex --marketquote=dot --markettype=trusted --ordertype=limit --orderside=ask --quantity=987345 --price=40 \
    --mrenclave $MRENCLAVE --direct """
    if proxy:
        accs = acc_arg(acc) + proxy_arg(proxy)
    else:
        accs = acc_arg(acc)
    market_args = base_arg(base) + quote_arg(quote) + markettype
    order_args = quantity_arg(quantity) + orderside_arg(side) + ordertype_arg(ordertype) + price_arg(price)
    ret = subprocess.run(cli + ["trusted", "place_order"] + accs + market_args + order_args + direct_tail(), stdout=subprocess.PIPE)
    print(ret.stdout)
    return ret.stdout.decode("utf-8").strip()

def direct_cancel_order(acc, proxy, base, quote, orderid):
    """ ./substratee-client -p 9994 -P 2094 trusted cancel_order --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
    --orderid=oijef03jaf --mrenclave $MRENCLAVE --direct """
    if proxy:
        accs = acc_arg(acc) + proxy_arg(proxy)
    else:
        accs = acc_arg(acc)
    market_args = base_arg(base) + quote_arg(quote) + markettype
    order_args = orderid_arg(orderid)

    ret = subprocess.run(cli + ["trusted", "place_order"] + accs + market_args + order_args + direct_tail(), stdout=subprocess.PIPE)
    print(ret.stdout)
    return ret.stdout.decode("utf-8").strip()

def direct_withdraw(acc, proxy, token, quantity):
    """ ./substratee-client -p 9994 -P 2094 trusted withdraw --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
    --tokenid=dot --quantity=293 --mrenclave $MRENCLAVE --direct """
    if proxy:
        print("Withdrawing " + str(quantity) + token + " from " + proxy)
        accs = acc_arg(acc) + proxy_arg(proxy)
    else:
        print("Withdrawing " + str(quantity) + token + " from " + acc)
        accs = acc_arg(acc)

    ret = subprocess.run(cli + ["trusted", "place_order"] + accs + quantity_arg(quantity) + token_arg(token) + direct_tail(), stdout=subprocess.PIPE)
    print(ret.stdout)
    return ret.stdout.decode("utf-8").strip()

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
    #register_proxy(alice, aliceIco)
    #register_proxy(bob, bobIco)

    #3 Alice deposits 100 tokenA
    deposit(alice, 500_000_000_000_000_000_000, tokenA)
    deposit(alice, 0, tokenB)

    #4 Bob deposits 100 tokenB
    deposit(bob, 500_000_000_000_000_000_000, tokenB)
    deposit(bob, 0, tokenA)

    #await_block() # wait some time to ensure enclave has read new block from main chain
    direct_get_balance(alice, tokenA)
    direct_get_balance(bob, tokenB)

    #5 Alice places a limit order selling 50 tokenA at a limit of 40 tokenB
    direct_place_order(alice, None, tokenA, tokenB, 'ask', 50_000_000_000_000_000_000, 'limit', 1_000_000_000_000_000_000)

    #6 Bob places a limit order buying 50 tokenA at a limit of 60 tokenB
    direct_place_order(bob, None, tokenA, tokenB, 'bid', 50_000_000_000_000_000_000, 'limit', 1_000_000_000_000_000_000)

    #7 The matching engine clears the match, sends it to the gateway
    #8 The gateway settles the match, publishes all details

    #await_block() # wait some time to matching engine had some time
    #9 The offchain balance of Alice is 50 tokenA plus 50 tokenB
    direct_get_balance(alice, tokenA)
    direct_get_balance(alice, tokenB)
    #10 The offchain balance of Bob is 50 tokenA plus 50 tokenB
    direct_get_balance(bob, tokenA)
    direct_get_balance(bob, tokenB)

    #11 Alice withdraws all her tokenB through direct call to gateway

    #12 Bob withdraws all his tokenA through indirect extrinsic

    #13 The offchain balance of Alice is zero tokenB and Bob is zero tokenA

    #14 The onchain balance of Alice is 50 tokenB and Bob is 50 tokenA
