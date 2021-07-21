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
import optparse

mrenclave_filename = "../bin/mrenclave.b58"
MRENCLAVE = ""
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
    if lines != ['number of workers registered: 0']:
        print("Reading MRENCLAVE from registered list")
        for line in lines:
            if "MRENCLAVE" in line:
                mrenclave = line.split()
                workers.append(mrenclave[1].strip())
    else:
        # open file instead
        print("Reading MRENCLAVE from file")
        with open(mrenclave_filename) as f:
            mrenclaves_with_line_ending = f.readlines()
            for mrenclave in mrenclaves_with_line_ending:
                workers.append(mrenclave.strip())

    MRENCLAVE = workers[0]
    print("Using mrenclave of first worker as default: " + MRENCLAVE)
    return workers

def await_block():
    subprocess.run(cli + ["listen", "-b", "1"], stdout=subprocess.PIPE)

def balance(acc):
    """ ./substratee-client -p 9994 -P 2094 balance //Alice """
    ret = subprocess.run(cli + ["balance"] + [acc], stdout=subprocess.PIPE)
    print("Onchain balance of " + acc + " " + str(int(ret.stdout)/PRECISION))
    return ret.stdout.decode("utf-8").strip()

def register_account(acc):
    """ ./substratee-client -p 9994 -P 2094 register-account //Alice """
    print("Registering " + acc)
    ret = subprocess.run(cli + ["register-account"] + [acc], stdout=subprocess.PIPE)
    await_block()
    return ret.stdout.decode("utf-8").strip()

def register_proxy(acc, proxy):
    """ ./substratee-client -p 9994 -P 2094 register-proxy //Alice //AliceIcognito"""
    print("Registering proxy account " + proxy + " for " + acc)
    ret = subprocess.run(cli + ["register-proxy"] + [acc] + [proxy], stdout=subprocess.PIPE)
    await_block()
    return ret.stdout.decode("utf-8").strip()

def deposit(acc, quantity, token):
    """ ./substratee-client -p 9994 -P 2094 deposit --accountid=//Alice --tokenid=polkadex --quantity=10000 """
    print("Deposit " + str(quantity/PRECISION) + " " + token + " to offchain account" + acc)
    ret = subprocess.run(cli + ["deposit"] + acc_arg(acc) + quantity_arg(quantity) + token_arg(token), stdout=subprocess.PIPE)
    await_block()
    return ret.stdout.decode("utf-8").strip()

def withdraw(acc, token, quantity):
    """  ./substratee-client -p 9994 -P 2094 withdraw --accountid=//Bob --tokenid=dot --quantity=1000 """
    print("Withdrawing " + str(quantity/PRECISION) + " " + token + " from " + acc)
    ret = subprocess.run(cli + ["withdraw"] + acc_arg(acc) + quantity_arg(quantity) + token_arg(token), stdout=subprocess.PIPE)
    print(ret.stdout.strip())
    await_block()
    return ret.stdout.decode("utf-8").strip()

def token_balance(acc, token):
    """ ./substratee-client -p 9994 -P 2094 token-balance //Alice btc"""
    ret = subprocess.run(cli + ["token-balance"] + [acc] + [token], stdout=subprocess.PIPE)
    print("Onchain balance of " + acc + " in " + token + ": " + str(int(ret.stdout)/PRECISION))
    return ret.stdout.decode("utf-8").strip()

def direct_get_balance(acc, token):
    """ ./substratee-client -p 9994 -P 2094 trusted get_balance --accountid=//AliceIncognito --tokenid=dot \
    --mrenclave $MRENCLAVE --direct
    """
    ret = subprocess.run(cli + ["trusted", "get_balance"] + acc_arg(acc) + token_arg(token) + direct_tail(), stdout=subprocess.PIPE)
    print("Offchain balance of " + acc + " " + str(int(ret.stdout)/PRECISION) + " " + token)
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
    --orderid=oijef03jaf --marketbase=polkadex --marketquote=dot --mrenclave $MRENCLAVE --direct """
    if proxy:
        accs = acc_arg(acc) + proxy_arg(proxy)
    else:
        accs = acc_arg(acc)
    market_args = base_arg(base) + quote_arg(quote) + markettype
    order_args = orderid_arg(orderid)

    ret = subprocess.run(cli + ["trusted", "cancel_order"] + accs + market_args + order_args + direct_tail(), stdout=subprocess.PIPE)
    print(ret.stdout)
    return ret.stdout.decode("utf-8").strip()

def direct_withdraw(acc, proxy, token, quantity):
    """ ./substratee-client -p 9994 -P 2094 trusted withdraw --accountid=//AliceIncognito --proxyaccountid=//AliceIncognitoProxy \
    --tokenid=dot --quantity=293 --mrenclave $MRENCLAVE --direct """
    if proxy:
        print("Withdrawing " + str(quantity/PRECISION) + " " + token + " from proxy" + proxy)
        accs = acc_arg(acc) + proxy_arg(proxy)
    else:
        print("Withdrawing " + str(quantity/PRECISION) + " " + token + " from " + acc)
        accs = acc_arg(acc)

    ret = subprocess.run(cli + ["trusted", "withdraw"] + accs + quantity_arg(quantity) + token_arg(token) + direct_tail(), stdout=subprocess.PIPE)
    print(ret.stdout)
    return ret.stdout.decode("utf-8").strip()