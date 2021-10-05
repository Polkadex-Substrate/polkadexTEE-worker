#!/usr/bin/env python3
import optparse
import subprocess
import time
import os

from polkadex_commands import *
alice = '//Alice'
bob = '//Bob'
aliceIco = '//AliceIcognito'
bobIco = '//BobIcognito'
btc = 'btc'
usd = 'usd'
UNIT = 1000000000000000000

def asset(first_value, second_value):
    if first_value == second_value:
        return True
    else:
        print("[ERROR] Value doesnt match" +  str(int(first_value)) + "!=" + str(int(second_value)))
        return False

def alice_offchain_balance(expectedUSDBalance, expectedBTCBalance):
    asset(int(direct_get_balance(alice, usd)), expectedUSDBalance)
    asset(int(direct_get_balance(alice, btc)), expectedBTCBalance)

def bob_offchain_balance(expectedUSDBalance, expectedBTCBalance):
    asset(int(direct_get_balance(bob, usd)), expectedUSDBalance)
    asset(int(direct_get_balance(bob, btc)), expectedBTCBalance)

def prRed(skk): print("\033[91m{}\033[00m" .format(skk))
def prGreen(skk): print("\033[92m{}\033[00m" .format(skk))
def prYellow(skk): print("\033[93m{}\033[00m" .format(skk))
def prPurple(skk): print("\033[95m{}\033[00m" .format(skk))
def prCyan(skk): print("\033[96m{}\033[00m" .format(skk))

if __name__ == '__main__':

    parser = optparse.OptionParser()
    parser.add_option('-p', '--node-port', dest='node', help='Node port', type=int)
    parser.add_option('-P', '--worker-port', dest='worker', help='Worker port', type=int)
    parser.add_option('-t', '--test-run', dest='test', help='indicates at which stage the test is currtly at', type=int)
    (options, args) = parser.parse_args()
    write_cli(options)

    try:
        os.remove("../bin/chain_relay_db.bin")
        os.remove("../bin/polkadex_storage/balance.bin")
        os.remove("../bin/polkadex_storage/nonce.bin")
        os.remove("../bin/polkadex_storage/orderbook.bin")
        os.remove("../bin/polkadex_storage/balance.bin.1")
        os.remove("../bin/polkadex_storage/nonce.bin.1")
        os.remove("../bin/polkadex_storage/orderbook.bin.1")
    except OSError:
        pass

    prYellow("[i] Starting Node and Worker")
    node = subprocess.Popen(["./polkadex-node", "--ws-port", str(options.node), "--dev", "--tmp"], stdout=open(os.devnull, 'wb'), stderr=open(os.devnull, 'wb'),  cwd="../../Polkadex/target/release")
    worker = subprocess.Popen(["./substratee-worker", "-P", str(options.worker), "-p", str(options.node), "-F", "8001/api/v2/ws", "-f", "127.0.0.1", "run"], stdout=open(os.devnull, 'wb'), stderr=open(os.devnull, 'wb'), cwd="../bin")

    prCyan("[-] Waiting 30 seconds to make sure everything is initialized")
    time.sleep(30)

    read_mrenclave()

    #ACCOUNT REGISTER
    register_account(alice)
    register_proxy(alice, aliceIco)
    register_account(bob)
    register_proxy(bob, bobIco)


    print("Checking balance of Alice onchain:")
    aliceBtc = token_balance(alice, btc)
    aliceUSD = token_balance(alice, usd)
    print("Alice USD Balance" + str(int(aliceUSD)/UNIT) + "BTC Balance" + str(int(aliceBtc)/UNIT))

    print("Checking balance of Bob onchain:")
    bobBtc = token_balance(bob, btc)
    bobUsd = token_balance(bob, usd)
    print("Alice USD Balance" + str(int(bobUsd)/UNIT) + "BTC Balance" + str(int(bobBtc)/UNIT))
    print("Check 1")

    #
    print("Start depositing funds from onchain to offchain:")
    # #3 Alice deposits 10 * UNIT BTC and 100 * UNIT USD
    deposit(alice, 10 * UNIT, btc)
    deposit(alice, 100 * UNIT, usd)
    deposit(bob, 10 * UNIT, btc)
    deposit(bob, 100 * UNIT, usd)
    await_block()

    print("Check balance")
    alice_offchain_balance(100 * UNIT, 10 * UNIT)
    bob_offchain_balance(100 * UNIT, 10 * UNIT)
    #
    print("Check new On-Chain Balance")
    asset(int(token_balance(alice, btc)), 0)
    asset(int(token_balance(alice, usd)), 0)
    asset(int(token_balance(bob, btc)), 0)
    asset(int(token_balance(bob, usd)), 0)

    prPurple("[T] Full State Recovery Test")

    prYellow("[i] Off-chain USD Balance for Alice/Bob:")
    alice_before_kill = direct_get_balance(alice, usd)
    bob_before_kill = direct_get_balance(bob, usd)
    print(alice_before_kill + "/" + bob_before_kill)

    prYellow("[i] Killing worker")
    worker.kill()

    prYellow("[i] Deleting Balance Storage To Make Sure It Comes From IPFS")
    try:
        os.remove("../bin/chain_relay_db.bin")
        os.remove("../bin/polkadex_storage/balance.bin")
        os.remove("../bin/polkadex_storage/balance.bin.1")
    except OSError:
        pass


    prYellow("[i] Starting worker back up")
    worker = subprocess.Popen(["./substratee-worker", "-P", str(options.worker), "-p", str(options.node), "-F", "8001/api/v2/ws", "-f", "127.0.0.1", "run"], stdout=open(os.devnull, 'wb'), stderr=open(os.devnull, 'wb'), cwd="../bin")
    prCyan("[-] Waiting 30 seconds to make sure everything is initialized")
    time.sleep(30)

    prYellow("[i] Off-chain USD Balance for Alice/Bob:")
    alice_after_kill = direct_get_balance(alice, usd)
    bob_after_kill = direct_get_balance(bob, usd)
    print(alice_after_kill + "/" + bob_after_kill)

    if (alice_before_kill, bob_before_kill) == (alice_after_kill, bob_after_kill):
        prGreen("[✓] Balances Recovered Successfully")
    else:
        prRed("[x] Balances Failed to Recover")

    prPurple("[E] End of Full State Recovery Test")

    # # Place Order BidLimit [A]
    await_block()
    uuid = direct_place_order(alice, None, btc, usd, 'bid', UNIT, 'limit', 2 * UNIT)
    await_block()
    alice_offchain_balance(98 * UNIT, 10 * UNIT)
    bob_offchain_balance(100 * UNIT, 10 * UNIT)
    uuid = direct_place_order(bob, None, btc, usd, 'ask', UNIT, 'limit', UNIT)
    await_block()
    await_block()
    alice_offchain_balance(98 * UNIT, 11 * UNIT)
    bob_offchain_balance(102 * UNIT, 9 * UNIT)

    print("Bob places Ask Limit [5 * UNIT,3 * UNIT]")
    await_block()
    uuid = direct_place_order(alice, None, btc, usd, 'ask', 3 * UNIT, 'limit', 5 * UNIT)
    await_block()
    alice_offchain_balance(98 * UNIT, 8 * UNIT)
    bob_offchain_balance(102 * UNIT, 9 * UNIT)
    await_block()

    uuid = direct_place_order(bob, None, btc, usd, 'bid', 2 * UNIT, 'limit', 7 * UNIT)
    await_block()
    alice_offchain_balance(108 * UNIT, 8 * UNIT)
    bob_offchain_balance(92 * UNIT, 11 * UNIT)
    await_block()

    uuid = direct_place_order(bob, None, btc, usd, 'bid', 2 * UNIT, 'limit', 6 * UNIT)
    await_block()
    alice_offchain_balance(113 * UNIT, 8 * UNIT)
    bob_offchain_balance(81 * UNIT, 12 * UNIT)
    await_block()

    uuid = direct_place_order(alice, None, btc, usd, 'ask', 1 * UNIT, 'limit', 2 * UNIT)
    await_block()
    alice_offchain_balance(119 * UNIT, 7 * UNIT)
    bob_offchain_balance(81 * UNIT, 13 * UNIT)
    await_block()

    uuid = direct_place_order(bob, None, btc, usd, 'bid', 4 * UNIT, 'limit', 5 * UNIT)
    await_block()
    alice_offchain_balance(119 * UNIT, 7 * UNIT)
    bob_offchain_balance(61 * UNIT, 13 * UNIT)
    await_block()

    uuid = direct_place_order(alice, None, btc, usd, 'ask', 2 * UNIT, 'limit', 2 * UNIT)
    await_block()
    alice_offchain_balance(129 * UNIT, 5 * UNIT)
    bob_offchain_balance(61 * UNIT, 15 * UNIT)
    await_block()

    uuid = direct_place_order(alice, None, btc, usd, 'ask', 1 * UNIT, 'market', 0)
    await_block()
    alice_offchain_balance(134 * UNIT, 4 * UNIT)
    bob_offchain_balance(61 * UNIT, 16 * UNIT)
    await_block()

    uuid = direct_place_order(alice, None, btc, usd, 'ask', 1 * UNIT, 'market', 0)
    await_block()
    alice_offchain_balance(139 * UNIT, 3 * UNIT)
    bob_offchain_balance(61 * UNIT, 17 * UNIT)
    await_block()

    #Test Cancel Orders

    uuid = direct_place_order(alice, None, btc, usd, 'bid', 2 * UNIT, 'limit', 2 * UNIT)
    await_block()
    print("Cancel order")
    result = direct_cancel_order(alice, None, tokenA, tokenB, uuid.strip('"'))
    print(str(result))
    await_block()

    worker.kill()
    node.kill()
