-- MariaDB dump 10.19  Distrib 10.5.9-MariaDB, for Linux (x86_64)
--
-- Host: 0.0.0.0    Database: finex_development
-- ------------------------------------------------------
-- Server version	5.7.26

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `accounts`
--

DROP TABLE IF EXISTS `accounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `accounts` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `member_id` int(11) NOT NULL,
  `currency_id` varchar(10) NOT NULL,
  `balance` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `locked` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_accounts_on_currency_id_and_member_id` (`currency_id`,`member_id`),
  KEY `index_accounts_on_member_id` (`member_id`)
) ENGINE=InnoDB AUTO_INCREMENT=40 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `accounts`
--

LOCK TABLES `accounts` WRITE;
/*!40000 ALTER TABLE `accounts` DISABLE KEYS */;
INSERT INTO `accounts` VALUES (1,1,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:59'),(2,1,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(3,1,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(4,1,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:59'),(5,2,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(6,2,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(7,2,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(8,2,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(9,3,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(10,3,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(11,3,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(12,3,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(13,4,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(14,4,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(15,4,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(16,4,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(17,5,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(18,5,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(19,5,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(20,5,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(21,6,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(22,6,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(23,6,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(24,6,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(25,7,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(26,7,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(27,7,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(28,7,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(29,8,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(30,8,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(31,8,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(32,8,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(33,9,'btc',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(34,9,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(35,9,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(36,9,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(37,10,'eth',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(38,10,'trst',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57'),(39,10,'usd',1000000000.0000000000000000,0.0000000000000000,'2021-05-20 14:43:57','2021-05-20 14:43:57');
/*!40000 ALTER TABLE `accounts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `adjustments`
--

DROP TABLE IF EXISTS `adjustments`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `adjustments` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `reason` varchar(255) NOT NULL,
  `description` text NOT NULL,
  `creator_id` bigint(20) NOT NULL,
  `validator_id` bigint(20) DEFAULT NULL,
  `amount` decimal(32,16) NOT NULL,
  `asset_account_code` smallint(5) unsigned NOT NULL,
  `receiving_account_number` varchar(64) NOT NULL,
  `currency_id` varchar(255) NOT NULL,
  `category` tinyint(4) NOT NULL,
  `state` tinyint(4) NOT NULL,
  `created_at` datetime(3) NOT NULL,
  `updated_at` datetime(3) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_adjustments_on_currency_id_and_state` (`currency_id`,`state`),
  KEY `index_adjustments_on_currency_id` (`currency_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `adjustments`
--

LOCK TABLES `adjustments` WRITE;
/*!40000 ALTER TABLE `adjustments` DISABLE KEYS */;
/*!40000 ALTER TABLE `adjustments` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `ar_internal_metadata`
--

DROP TABLE IF EXISTS `ar_internal_metadata`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `ar_internal_metadata` (
  `key` varchar(255) NOT NULL,
  `value` varchar(255) DEFAULT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `ar_internal_metadata`
--

LOCK TABLES `ar_internal_metadata` WRITE;
/*!40000 ALTER TABLE `ar_internal_metadata` DISABLE KEYS */;
INSERT INTO `ar_internal_metadata` VALUES ('environment','development','2021-03-24 08:03:13','2021-03-24 08:03:13');
/*!40000 ALTER TABLE `ar_internal_metadata` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `assets`
--

DROP TABLE IF EXISTS `assets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `assets` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `code` int(11) NOT NULL,
  `currency_id` varchar(255) NOT NULL,
  `reference_type` varchar(255) DEFAULT NULL,
  `reference_id` int(11) DEFAULT NULL,
  `debit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `credit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_assets_on_currency_id` (`currency_id`),
  KEY `index_assets_on_reference_type_and_reference_id` (`reference_type`,`reference_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `assets`
--

LOCK TABLES `assets` WRITE;
/*!40000 ALTER TABLE `assets` DISABLE KEYS */;
/*!40000 ALTER TABLE `assets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `beneficiaries`
--

DROP TABLE IF EXISTS `beneficiaries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `beneficiaries` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `member_id` bigint(20) NOT NULL,
  `currency_id` varchar(10) NOT NULL,
  `name` varchar(64) NOT NULL,
  `description` varchar(255) DEFAULT '',
  `data_encrypted` varchar(1024) DEFAULT NULL,
  `pin` mediumint(8) unsigned NOT NULL,
  `sent_at` datetime DEFAULT NULL,
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_beneficiaries_on_currency_id` (`currency_id`),
  KEY `index_beneficiaries_on_member_id` (`member_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `beneficiaries`
--

LOCK TABLES `beneficiaries` WRITE;
/*!40000 ALTER TABLE `beneficiaries` DISABLE KEYS */;
/*!40000 ALTER TABLE `beneficiaries` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `blockchains`
--

DROP TABLE IF EXISTS `blockchains`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `blockchains` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `key` varchar(255) NOT NULL,
  `name` varchar(255) DEFAULT NULL,
  `client` varchar(255) NOT NULL,
  `server` varchar(255) DEFAULT NULL,
  `height` bigint(20) NOT NULL,
  `explorer_address` varchar(255) DEFAULT NULL,
  `explorer_transaction` varchar(255) DEFAULT NULL,
  `min_confirmations` int(11) NOT NULL DEFAULT '6',
  `status` varchar(255) NOT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_blockchains_on_key` (`key`),
  KEY `index_blockchains_on_status` (`status`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `blockchains`
--

LOCK TABLES `blockchains` WRITE;
/*!40000 ALTER TABLE `blockchains` DISABLE KEYS */;
INSERT INTO `blockchains` VALUES (1,'prt-kovan','Ethereum Kovan','parity','http://127.0.0.1:8545',2500000,'https://kovan.etherscan.io/address/#{address}','https://kovan.etherscan.io/tx/#{txid}',6,'disabled','2021-03-24 08:03:14','2021-03-24 08:03:14'),(2,'eth-rinkeby','Ethereum Rinkeby','geth','http://127.0.0.1:8545',4000000,'https://rinkeby.etherscan.io/address/#{address}','https://rinkeby.etherscan.io/tx/#{txid}',6,'active','2021-03-24 08:03:14','2021-03-24 08:03:15'),(3,'eth-mainet','Ethereum Mainet','geth','http://127.0.0.1:8545',7500000,'https://etherscan.io/address/#{address}','https://etherscan.io/tx/#{txid}',6,'disabled','2021-03-24 08:03:14','2021-03-24 08:03:15'),(4,'btc-testnet','Bitcoin Testnet','bitcoin','http://user:password@127.0.0.1:18332',1500000,'https://testnet.blockchain.info/address/#{address}','https://testnet.blockchain.info/tx/#{txid}',6,'active','2021-03-24 08:03:14','2021-03-24 08:03:14');
/*!40000 ALTER TABLE `blockchains` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `currencies`
--

DROP TABLE IF EXISTS `currencies`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `currencies` (
  `id` varchar(10) NOT NULL,
  `name` varchar(255) DEFAULT NULL,
  `description` text,
  `homepage` varchar(255) DEFAULT NULL,
  `blockchain_key` varchar(32) DEFAULT NULL,
  `parent_id` varchar(255) DEFAULT NULL,
  `type` varchar(30) NOT NULL DEFAULT 'coin',
  `deposit_fee` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `min_deposit_amount` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `min_collection_amount` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `withdraw_fee` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `min_withdraw_amount` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `withdraw_limit_24h` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `withdraw_limit_72h` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `position` int(11) NOT NULL,
  `options` json DEFAULT NULL,
  `visible` tinyint(1) NOT NULL DEFAULT '1',
  `deposit_enabled` tinyint(1) NOT NULL DEFAULT '1',
  `withdrawal_enabled` tinyint(1) NOT NULL DEFAULT '1',
  `base_factor` bigint(20) NOT NULL DEFAULT '1',
  `precision` tinyint(4) NOT NULL DEFAULT '8',
  `icon_url` varchar(255) DEFAULT NULL,
  `price` decimal(32,16) NOT NULL DEFAULT '1.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_currencies_on_parent_id` (`parent_id`),
  KEY `index_currencies_on_position` (`position`),
  KEY `index_currencies_on_visible` (`visible`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `currencies`
--

LOCK TABLES `currencies` WRITE;
/*!40000 ALTER TABLE `currencies` DISABLE KEYS */;
INSERT INTO `currencies` VALUES ('btc','Bitcoin',NULL,NULL,'btc-testnet',NULL,'coin',0.0000000000000000,0.0000356000000000,0.0000356000000000,0.0000000000000000,0.0000000000000000,0.1000000000000000,0.2000000000000000,2,'{}',1,1,1,100000000,8,NULL,1.0000000000000000,'2021-03-24 08:03:14','2021-03-24 08:03:14'),('eth','Ethereum',NULL,NULL,'eth-rinkeby',NULL,'coin',0.0000000000000000,0.0002100000000000,0.0002100000000000,0.0000000000000000,0.0000000000000000,0.2000000000000000,0.5000000000000001,3,'{\"gas_limit\": 21000, \"gas_price\": 1000000000}',1,1,1,1000000000000000000,8,NULL,1.0000000000000000,'2021-03-24 08:03:14','2021-03-24 08:03:14'),('trst','WeTrust',NULL,NULL,'eth-rinkeby','eth','coin',0.0000000000000000,2.0000000000000000,2.0000000000000000,0.0000000000000000,0.0000000000000000,300.0000000000000000,600.0000000000000000,4,'{\"gas_limit\": 90000, \"gas_price\": 1000000000, \"erc20_contract_address\": \"0x0000000000000000000000000000000000000000\"}',1,1,1,1000000,8,NULL,1.0000000000000000,'2021-03-24 08:03:14','2021-03-24 08:03:14'),('usd','US Dollar',NULL,NULL,NULL,NULL,'fiat',0.0000000000000000,0.0000000000000000,0.0000000000000000,0.0000000000000000,0.0000000000000000,100.0000000000000000,200.0000000000000000,1,'{}',1,1,1,1,2,NULL,1.0000000000000000,'2021-03-24 08:03:14','2021-03-24 08:03:14');
/*!40000 ALTER TABLE `currencies` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `currencies_wallets`
--

DROP TABLE IF EXISTS `currencies_wallets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `currencies_wallets` (
  `currency_id` varchar(255) DEFAULT NULL,
  `wallet_id` int(11) DEFAULT NULL,
  UNIQUE KEY `index_currencies_wallets_on_currency_id_and_wallet_id` (`currency_id`,`wallet_id`),
  KEY `index_currencies_wallets_on_currency_id` (`currency_id`),
  KEY `index_currencies_wallets_on_wallet_id` (`wallet_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `currencies_wallets`
--

LOCK TABLES `currencies_wallets` WRITE;
/*!40000 ALTER TABLE `currencies_wallets` DISABLE KEYS */;
INSERT INTO `currencies_wallets` VALUES ('btc',5),('btc',6),('btc',7),('eth',1),('eth',2),('eth',3),('eth',4),('trst',1),('trst',2),('trst',3);
/*!40000 ALTER TABLE `currencies_wallets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `deposits`
--

DROP TABLE IF EXISTS `deposits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `deposits` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `member_id` int(11) NOT NULL,
  `currency_id` varchar(10) NOT NULL,
  `amount` decimal(32,16) NOT NULL,
  `fee` decimal(32,16) NOT NULL,
  `address` varchar(95) DEFAULT NULL,
  `from_addresses` text,
  `txid` varchar(128) CHARACTER SET utf8 COLLATE utf8_bin DEFAULT NULL,
  `txout` int(11) DEFAULT NULL,
  `aasm_state` varchar(30) NOT NULL,
  `block_number` int(11) DEFAULT NULL,
  `type` varchar(30) NOT NULL,
  `transfer_type` int(11) DEFAULT NULL,
  `tid` varchar(64) CHARACTER SET utf8 COLLATE utf8_bin NOT NULL,
  `spread` varchar(1000) DEFAULT NULL,
  `created_at` datetime(3) NOT NULL,
  `updated_at` datetime(3) NOT NULL,
  `completed_at` datetime(3) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_deposits_on_currency_id_and_txid_and_txout` (`currency_id`,`txid`,`txout`),
  KEY `index_deposits_on_aasm_state_and_member_id_and_currency_id` (`aasm_state`,`member_id`,`currency_id`),
  KEY `index_deposits_on_currency_id` (`currency_id`),
  KEY `index_deposits_on_member_id_and_txid` (`member_id`,`txid`),
  KEY `index_deposits_on_tid` (`tid`),
  KEY `index_deposits_on_type` (`type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `deposits`
--

LOCK TABLES `deposits` WRITE;
/*!40000 ALTER TABLE `deposits` DISABLE KEYS */;
/*!40000 ALTER TABLE `deposits` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `engines`
--

DROP TABLE IF EXISTS `engines`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `engines` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `driver` varchar(255) NOT NULL,
  `uid` varchar(255) DEFAULT NULL,
  `url` varchar(255) DEFAULT NULL,
  `key_encrypted` varchar(255) DEFAULT NULL,
  `secret_encrypted` varchar(255) DEFAULT NULL,
  `data_encrypted` varchar(1024) DEFAULT NULL,
  `state` int(11) NOT NULL DEFAULT '1',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `engines`
--

LOCK TABLES `engines` WRITE;
/*!40000 ALTER TABLE `engines` DISABLE KEYS */;
INSERT INTO `engines` VALUES (1,'local-finex-engine','finex-spot','NULL','NULL','NULL','NULL','{}',1),(2,'peatio-default-engine','peatio','NULL','NULL','NULL','NULL','{}',1),(3,'opendax-finex-engine','opendax','U313725406','url','key','secret','{}',1),(4,'bitfinex-finex-engine','bitfinex','U313725406','url','key','secret','{\"ws_url\":\"ws://localhost:4444/ws/bitfinex\",\"rest_url\":\"http://localhost:4444/rest/bitfinex\"}',1),(5,'binance-finex-engine','binance','U313725406','http://localhost:4444/rest/binance','key','secret','{}',1),(6,'aggregated-finex-engine','aggregated','U313725406','','key','secret','{\"engines\":[{\"driver\":\"binance\",\"url\":\"http://localhost:4444/rest/binance\"},{\"driver\":\"bitfinex\",\"data\":{\"ws_url\":\"ws://localhost:4444/ws/bitfinex\",\"rest_url\":\"http://localhost:4444/rest/bitfinex\"}}]}',1),(7,'cryptocom-finex-engine','cryptocom','U313725406','','key','secret','{\"ws_url\":\"ws://localhost:4444/ws/cryptocom\",\"rest_url\":\"http://localhost:4444/rest/cryptocom\"}',1);
/*!40000 ALTER TABLE `engines` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `expenses`
--

DROP TABLE IF EXISTS `expenses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `expenses` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `code` int(11) NOT NULL,
  `currency_id` varchar(255) NOT NULL,
  `reference_type` varchar(255) DEFAULT NULL,
  `reference_id` int(11) DEFAULT NULL,
  `debit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `credit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_expenses_on_currency_id` (`currency_id`),
  KEY `index_expenses_on_reference_type_and_reference_id` (`reference_type`,`reference_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `expenses`
--

LOCK TABLES `expenses` WRITE;
/*!40000 ALTER TABLE `expenses` DISABLE KEYS */;
/*!40000 ALTER TABLE `expenses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `internal_transfers`
--

DROP TABLE IF EXISTS `internal_transfers`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `internal_transfers` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `currency_id` varchar(255) NOT NULL,
  `amount` decimal(32,16) NOT NULL,
  `sender_id` bigint(20) NOT NULL,
  `receiver_id` bigint(20) NOT NULL,
  `state` int(11) NOT NULL DEFAULT '1',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `internal_transfers`
--

LOCK TABLES `internal_transfers` WRITE;
/*!40000 ALTER TABLE `internal_transfers` DISABLE KEYS */;
/*!40000 ALTER TABLE `internal_transfers` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `jobs`
--

DROP TABLE IF EXISTS `jobs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `jobs` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `pointer` int(10) unsigned DEFAULT NULL,
  `counter` int(11) DEFAULT NULL,
  `data` json DEFAULT NULL,
  `error_code` tinyint(3) unsigned NOT NULL DEFAULT '255',
  `error_message` varchar(255) DEFAULT NULL,
  `started_at` datetime DEFAULT NULL,
  `finished_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `jobs`
--

LOCK TABLES `jobs` WRITE;
/*!40000 ALTER TABLE `jobs` DISABLE KEYS */;
/*!40000 ALTER TABLE `jobs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `liabilities`
--

DROP TABLE IF EXISTS `liabilities`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `liabilities` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `code` int(11) NOT NULL,
  `currency_id` varchar(255) NOT NULL,
  `member_id` int(11) DEFAULT NULL,
  `reference_type` varchar(255) DEFAULT NULL,
  `reference_id` int(11) DEFAULT NULL,
  `debit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `credit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_liabilities_on_currency_id` (`currency_id`),
  KEY `index_liabilities_on_member_id` (`member_id`),
  KEY `index_liabilities_on_reference_type_and_reference_id` (`reference_type`,`reference_id`)
) ENGINE=InnoDB AUTO_INCREMENT=21 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `liabilities`
--

LOCK TABLES `liabilities` WRITE;
/*!40000 ALTER TABLE `liabilities` DISABLE KEYS */;
INSERT INTO `liabilities` VALUES (1,202,'btc',1,'Order',1,1.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(2,212,'btc',1,'Order',1,0.0000000000000000,1.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(3,202,'btc',1,'Order',2,2.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(4,212,'btc',1,'Order',2,0.0000000000000000,2.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(5,202,'btc',1,'Order',3,3.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(6,212,'btc',1,'Order',3,0.0000000000000000,3.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(7,201,'usd',1,'Order',4,5.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(8,211,'usd',1,'Order',4,0.0000000000000000,5.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(9,201,'usd',1,'Order',5,10.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(10,211,'usd',1,'Order',5,0.0000000000000000,10.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(11,212,'btc',1,'Order',2,2.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(12,202,'btc',1,'Order',2,0.0000000000000000,2.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(13,211,'usd',1,'Order',4,5.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(14,201,'usd',1,'Order',4,0.0000000000000000,5.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(15,211,'usd',1,'Order',5,10.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(16,201,'usd',1,'Order',5,0.0000000000000000,10.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(17,212,'btc',1,'Order',1,1.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(18,202,'btc',1,'Order',1,0.0000000000000000,1.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(19,212,'btc',1,'Order',3,3.0000000000000000,0.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59'),(20,202,'btc',1,'Order',3,0.0000000000000000,3.0000000000000000,'2021-05-20 14:43:59','2021-05-20 14:43:59');
/*!40000 ALTER TABLE `liabilities` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `markets`
--

DROP TABLE IF EXISTS `markets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `markets` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `symbol` varchar(20) NOT NULL,
  `type` varchar(255) NOT NULL DEFAULT 'spot',
  `base_unit` varchar(10) NOT NULL,
  `quote_unit` varchar(10) NOT NULL,
  `engine_id` bigint(20) NOT NULL,
  `amount_precision` tinyint(4) NOT NULL DEFAULT '4',
  `price_precision` tinyint(4) NOT NULL DEFAULT '4',
  `min_price` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `max_price` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `min_amount` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `position` int(11) NOT NULL,
  `data` json DEFAULT NULL,
  `state` varchar(32) NOT NULL DEFAULT 'enabled',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_markets_on_base_unit_and_quote_unit_and_type` (`base_unit`,`quote_unit`,`type`),
  UNIQUE KEY `index_markets_on_symbol_and_type` (`symbol`,`type`),
  KEY `index_markets_on_base_unit` (`base_unit`),
  KEY `index_markets_on_engine_id` (`engine_id`),
  KEY `index_markets_on_position` (`position`),
  KEY `index_markets_on_quote_unit` (`quote_unit`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `markets`
--

LOCK TABLES `markets` WRITE;
/*!40000 ALTER TABLE `markets` DISABLE KEYS */;
INSERT INTO `markets` VALUES (1,'btcusd','spot','btc','usd',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,100,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58'),(2,'ethbtc','spot','eth','btc',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,103,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58'),(3,'ethusd','spot','eth','usd',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,101,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58'),(4,'trstbtc','spot','trst','btc',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,104,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58'),(5,'trsteth','spot','trst','eth',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,105,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58'),(6,'trstusd','spot','trst','usd',1,4,4,0.0001000000000000,0.0000000000000000,0.0001000000000000,102,'{}','enabled','2021-05-20 14:43:58','2021-05-20 14:43:58');
/*!40000 ALTER TABLE `markets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `members`
--

DROP TABLE IF EXISTS `members`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `members` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `uid` varchar(32) NOT NULL,
  `email` varchar(255) NOT NULL,
  `level` int(11) NOT NULL,
  `role` varchar(16) NOT NULL,
  `group` varchar(32) NOT NULL DEFAULT 'vip-0',
  `state` varchar(16) NOT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  `username` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_members_on_email` (`email`),
  UNIQUE KEY `index_members_on_uid` (`uid`),
  UNIQUE KEY `index_members_on_username` (`username`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `members`
--

LOCK TABLES `members` WRITE;
/*!40000 ALTER TABLE `members` DISABLE KEYS */;
INSERT INTO `members` VALUES (1,'U487205863','jefferson@crist.biz',3,'bench','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Johnny'),(2,'U180977303','napoleonreinger@goyettehahn.com',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Tiffany'),(3,'U835000656','carmeloerdman@herman.io',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Billy'),(4,'U260596984','sonny@bogisich.io',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Roy'),(5,'U876658599','jermaine@breitenberglangosh.io',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Lily'),(6,'U447324758','yolanda@kulas.co',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Ron'),(7,'U408283129','arnulfo@oconnellgusikowski.com',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Harry'),(8,'U271077155','vivienpadberg@barton.net',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Larry'),(9,'U274470943','josueluettgen@hegmann.net',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Woow'),(10,'U313725406','james@lindgren.co',3,'member','vip-0','active','2021-05-20 14:43:58','2021-05-20 14:43:58','Thomas');
/*!40000 ALTER TABLE `members` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `operations_accounts`
--

DROP TABLE IF EXISTS `operations_accounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `operations_accounts` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `code` mediumint(9) NOT NULL,
  `type` varchar(10) NOT NULL,
  `kind` varchar(30) NOT NULL,
  `currency_type` varchar(10) NOT NULL,
  `description` varchar(100) DEFAULT NULL,
  `scope` varchar(10) NOT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_operations_accounts_on_code` (`code`),
  UNIQUE KEY `index_operations_accounts_on_type_and_kind_and_currency_type` (`type`,`kind`,`currency_type`),
  KEY `index_operations_accounts_on_currency_type` (`currency_type`),
  KEY `index_operations_accounts_on_scope` (`scope`),
  KEY `index_operations_accounts_on_type` (`type`)
) ENGINE=InnoDB AUTO_INCREMENT=11 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `operations_accounts`
--

LOCK TABLES `operations_accounts` WRITE;
/*!40000 ALTER TABLE `operations_accounts` DISABLE KEYS */;
INSERT INTO `operations_accounts` VALUES (1,101,'asset','main','fiat','Main Fiat Assets Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14'),(2,102,'asset','main','coin','Main Crypto Assets Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14'),(3,201,'liability','main','fiat','Main Fiat Liabilities Account','member','2021-03-24 08:03:14','2021-03-24 08:03:14'),(4,202,'liability','main','coin','Main Crypto Liabilities Account','member','2021-03-24 08:03:14','2021-03-24 08:03:14'),(5,211,'liability','locked','fiat','Locked Fiat Liabilities Account','member','2021-03-24 08:03:14','2021-03-24 08:03:14'),(6,212,'liability','locked','coin','Locked Crypto Liabilities Account','member','2021-03-24 08:03:14','2021-03-24 08:03:14'),(7,301,'revenue','main','fiat','Main Fiat Revenues Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14'),(8,302,'revenue','main','coin','Main Crypto Revenues Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14'),(9,401,'expense','main','fiat','Main Fiat Expenses Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14'),(10,402,'expense','main','coin','Main Crypto Expenses Account','platform','2021-03-24 08:03:14','2021-03-24 08:03:14');
/*!40000 ALTER TABLE `operations_accounts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `orders`
--

DROP TABLE IF EXISTS `orders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `orders` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `uuid` varbinary(16) NOT NULL,
  `remote_id` varchar(255) DEFAULT NULL,
  `bid` varchar(10) NOT NULL,
  `ask` varchar(10) NOT NULL,
  `market_id` varchar(20) NOT NULL,
  `market_type` varchar(255) NOT NULL DEFAULT 'spot',
  `price` decimal(32,16) DEFAULT NULL,
  `volume` decimal(32,16) NOT NULL,
  `origin_volume` decimal(32,16) NOT NULL,
  `maker_fee` decimal(17,16) NOT NULL DEFAULT '0.0000000000000000',
  `taker_fee` decimal(17,16) NOT NULL DEFAULT '0.0000000000000000',
  `state` int(11) NOT NULL,
  `type` varchar(8) NOT NULL,
  `member_id` int(11) NOT NULL,
  `ord_type` varchar(30) NOT NULL,
  `locked` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `origin_locked` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `funds_received` decimal(32,16) DEFAULT '0.0000000000000000',
  `trades_count` int(11) NOT NULL DEFAULT '0',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_orders_on_uuid` (`uuid`),
  KEY `index_orders_on_member_id` (`member_id`),
  KEY `index_orders_on_state` (`state`),
  KEY `index_orders_on_type_and_market_id_and_market_type` (`type`,`market_id`,`market_type`),
  KEY `index_orders_on_type_and_member_id` (`type`,`member_id`),
  KEY `index_orders_on_type_and_state_and_market_id_and_market_type` (`type`,`state`,`market_id`,`market_type`),
  KEY `index_orders_on_type_and_state_and_member_id` (`type`,`state`,`member_id`),
  KEY `index_orders_on_updated_at` (`updated_at`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `orders`
--

LOCK TABLES `orders` WRITE;
/*!40000 ALTER TABLE `orders` DISABLE KEYS */;
INSERT INTO `orders` VALUES (1,'Ï¦µ¢¹yë´»1¿˜4°',NULL,'usd','btc','btcusd','spot',10.0000000000000000,1.0000000000000000,1.0000000000000000,0.0010000000000000,0.0020000000000000,-100,'OrderAsk',1,'limit',1.0000000000000000,1.0000000000000000,0.0000000000000000,0,'2021-05-20 17:43:59','2021-05-20 14:43:59'),(2,'Ï§Iv¹yë´»1¿˜4°',NULL,'usd','btc','btcusd','spot',10.0000000000000000,2.0000000000000000,2.0000000000000000,0.0010000000000000,0.0020000000000000,-100,'OrderAsk',1,'limit',2.0000000000000000,2.0000000000000000,0.0000000000000000,0,'2021-05-20 17:43:59','2021-05-20 14:43:59'),(3,'Ï¨PM¹yë´»1¿˜4°',NULL,'usd','btc','btcusd','spot',10.0000000000000000,3.0000000000000000,3.0000000000000000,0.0010000000000000,0.0020000000000000,-100,'OrderAsk',1,'limit',3.0000000000000000,3.0000000000000000,0.0000000000000000,0,'2021-05-20 17:43:59','2021-05-20 14:43:59'),(4,'Ï¨ìR¹yë´»1¿˜4°',NULL,'usd','btc','btcusd','spot',5.0000000000000000,1.0000000000000000,1.0000000000000000,0.0010000000000000,0.0020000000000000,-100,'OrderBid',1,'limit',5.0000000000000000,5.0000000000000000,0.0000000000000000,0,'2021-05-20 17:43:59','2021-05-20 14:43:59'),(5,'Ï©b¸¹yë´»1¿˜4°',NULL,'usd','btc','btcusd','spot',5.0000000000000000,2.0000000000000000,2.0000000000000000,0.0010000000000000,0.0020000000000000,-100,'OrderBid',1,'limit',10.0000000000000000,10.0000000000000000,0.0000000000000000,0,'2021-05-20 17:43:59','2021-05-20 14:43:59');
/*!40000 ALTER TABLE `orders` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `payment_addresses`
--

DROP TABLE IF EXISTS `payment_addresses`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `payment_addresses` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `member_id` bigint(20) DEFAULT NULL,
  `wallet_id` bigint(20) DEFAULT NULL,
  `address` varchar(95) DEFAULT NULL,
  `remote` tinyint(1) NOT NULL DEFAULT '0',
  `secret_encrypted` varchar(255) DEFAULT NULL,
  `details_encrypted` varchar(1024) DEFAULT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_payment_addresses_on_member_id` (`member_id`),
  KEY `index_payment_addresses_on_wallet_id` (`wallet_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `payment_addresses`
--

LOCK TABLES `payment_addresses` WRITE;
/*!40000 ALTER TABLE `payment_addresses` DISABLE KEYS */;
/*!40000 ALTER TABLE `payment_addresses` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `refunds`
--

DROP TABLE IF EXISTS `refunds`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `refunds` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `deposit_id` bigint(20) NOT NULL,
  `state` varchar(30) NOT NULL,
  `address` varchar(255) NOT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_refunds_on_deposit_id` (`deposit_id`),
  KEY `index_refunds_on_state` (`state`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `refunds`
--

LOCK TABLES `refunds` WRITE;
/*!40000 ALTER TABLE `refunds` DISABLE KEYS */;
/*!40000 ALTER TABLE `refunds` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `revenues`
--

DROP TABLE IF EXISTS `revenues`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `revenues` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `code` int(11) NOT NULL,
  `currency_id` varchar(255) NOT NULL,
  `member_id` int(11) DEFAULT NULL,
  `reference_type` varchar(255) DEFAULT NULL,
  `reference_id` int(11) DEFAULT NULL,
  `debit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `credit` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_revenues_on_currency_id` (`currency_id`),
  KEY `index_revenues_on_reference_type_and_reference_id` (`reference_type`,`reference_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `revenues`
--

LOCK TABLES `revenues` WRITE;
/*!40000 ALTER TABLE `revenues` DISABLE KEYS */;
/*!40000 ALTER TABLE `revenues` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `schema_migrations`
--

DROP TABLE IF EXISTS `schema_migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `schema_migrations` (
  `version` varchar(255) NOT NULL,
  PRIMARY KEY (`version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `schema_migrations`
--

LOCK TABLES `schema_migrations` WRITE;
/*!40000 ALTER TABLE `schema_migrations` DISABLE KEYS */;
INSERT INTO `schema_migrations` VALUES ('20180112151205'),('20180212115002'),('20180212115751'),('20180213160501'),('20180215124645'),('20180215131129'),('20180215144645'),('20180215144646'),('20180216145412'),('20180227163417'),('20180303121013'),('20180303211737'),('20180305111648'),('20180315132521'),('20180315145436'),('20180315150348'),('20180315185255'),('20180325001828'),('20180327020701'),('20180329145257'),('20180329145557'),('20180329154130'),('20180403115050'),('20180403134930'),('20180403135744'),('20180403145234'),('20180403231931'),('20180406080444'),('20180406185130'),('20180407082641'),('20180409115144'),('20180409115902'),('20180416160438'),('20180417085823'),('20180417111305'),('20180417175453'),('20180419122223'),('20180425094920'),('20180425152420'),('20180425224307'),('20180501082703'),('20180501141718'),('20180516094307'),('20180516101606'),('20180516104042'),('20180516105035'),('20180516110336'),('20180516124235'),('20180516131005'),('20180516133138'),('20180517084245'),('20180517101842'),('20180517110003'),('20180522105709'),('20180522121046'),('20180522165830'),('20180524170927'),('20180525101406'),('20180529125011'),('20180530122201'),('20180605104154'),('20180613140856'),('20180613144712'),('20180704103131'),('20180704115110'),('20180708014826'),('20180708171446'),('20180716115113'),('20180718113111'),('20180719123616'),('20180719172203'),('20180720165705'),('20180726110440'),('20180727054453'),('20180803144827'),('20180808144704'),('20180813105100'),('20180905112301'),('20180925123806'),('20181004114428'),('20181017114624'),('20181027192001'),('20181028000150'),('20181105102116'),('20181105102422'),('20181105102537'),('20181105120211'),('20181120113445'),('20181126101312'),('20181210162905'),('20181219115439'),('20181219133822'),('20181226170925'),('20181229051129'),('20190110164859'),('20190115165813'),('20190116140939'),('20190204142656'),('20190213104708'),('20190225171726'),('20190401121727'),('20190402130148'),('20190426145506'),('20190502103256'),('20190529142209'),('20190617090551'),('20190624102330'),('20190711114027'),('20190723202251'),('20190725131843'),('20190726161540'),('20190807092706'),('20190813121822'),('20190814102636'),('20190816125948'),('20190829035814'),('20190829152927'),('20190830082950'),('20190902134819'),('20190902141139'),('20190904143050'),('20190905050444'),('20190910105717'),('20190923085927'),('20200117160600'),('20200211124707'),('20200220133250'),('20200305140516'),('20200316132213'),('20200317080916'),('20200414155144'),('20200420141636'),('20200504183201'),('20200513153429'),('20200527130534'),('20200603164002'),('20200622185615'),('20200728143753'),('20200804091304'),('20200805102000'),('20200805102001'),('20200805102002'),('20200805144308'),('20200806143442'),('20200824172823'),('20200826091118'),('20200902082403'),('20200903113109'),('20200907133518'),('20200908105929'),('20200909083000'),('20201001094156'),('20201118151056'),('20201204134602'),('20201206205429'),('20201207134745'),('20201222155655'),('20210112063704'),('20210120135842'),('20210128083207'),('20210210133912'),('20210219144535'),('20210225123519'),('20210302120855');
/*!40000 ALTER TABLE `schema_migrations` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `stats_member_pnl`
--

DROP TABLE IF EXISTS `stats_member_pnl`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `stats_member_pnl` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `member_id` int(11) NOT NULL,
  `pnl_currency_id` varchar(10) NOT NULL,
  `currency_id` varchar(10) NOT NULL,
  `total_credit` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_credit_fees` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_debit_fees` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_debit` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_credit_value` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_debit_value` decimal(48,16) DEFAULT '0.0000000000000000',
  `total_balance_value` decimal(48,16) DEFAULT '0.0000000000000000',
  `average_balance_price` decimal(48,16) DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_currency_ids_and_member_id` (`pnl_currency_id`,`currency_id`,`member_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `stats_member_pnl`
--

LOCK TABLES `stats_member_pnl` WRITE;
/*!40000 ALTER TABLE `stats_member_pnl` DISABLE KEYS */;
/*!40000 ALTER TABLE `stats_member_pnl` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `stats_member_pnl_idx`
--

DROP TABLE IF EXISTS `stats_member_pnl_idx`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `stats_member_pnl_idx` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `pnl_currency_id` varchar(10) NOT NULL,
  `currency_id` varchar(10) NOT NULL,
  `reference_type` varchar(255) NOT NULL,
  `last_id` bigint(20) DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_currency_ids_and_type` (`pnl_currency_id`,`currency_id`,`reference_type`),
  KEY `index_currency_ids_and_last_id` (`pnl_currency_id`,`currency_id`,`last_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `stats_member_pnl_idx`
--

LOCK TABLES `stats_member_pnl_idx` WRITE;
/*!40000 ALTER TABLE `stats_member_pnl_idx` DISABLE KEYS */;
/*!40000 ALTER TABLE `stats_member_pnl_idx` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `trades`
--

DROP TABLE IF EXISTS `trades`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `trades` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `price` decimal(32,16) NOT NULL,
  `amount` decimal(32,16) NOT NULL,
  `total` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `maker_order_id` int(11) NOT NULL,
  `taker_order_id` int(11) NOT NULL,
  `market_id` varchar(20) NOT NULL,
  `market_type` varchar(255) NOT NULL DEFAULT 'spot',
  `maker_id` int(11) NOT NULL,
  `taker_id` int(11) NOT NULL,
  `taker_type` varchar(20) NOT NULL DEFAULT '',
  `created_at` datetime(3) NOT NULL,
  `updated_at` datetime(3) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_trades_on_created_at` (`created_at`),
  KEY `index_trades_on_maker_id_and_market_type_and_created_at` (`maker_id`,`market_type`,`created_at`),
  KEY `index_trades_on_maker_id_and_market_type` (`maker_id`,`market_type`),
  KEY `index_trades_on_maker_id` (`maker_id`),
  KEY `index_trades_on_maker_order_id` (`maker_order_id`),
  KEY `index_trades_on_taker_id_and_market_type` (`taker_id`,`market_type`),
  KEY `index_trades_on_taker_order_id` (`taker_order_id`),
  KEY `index_trades_on_taker_type` (`taker_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `trades`
--

LOCK TABLES `trades` WRITE;
/*!40000 ALTER TABLE `trades` DISABLE KEYS */;
/*!40000 ALTER TABLE `trades` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `trading_fees`
--

DROP TABLE IF EXISTS `trading_fees`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `trading_fees` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `market_id` varchar(20) NOT NULL DEFAULT 'any',
  `market_type` varchar(255) NOT NULL DEFAULT 'spot',
  `group` varchar(32) NOT NULL DEFAULT 'any',
  `maker` decimal(7,6) NOT NULL DEFAULT '0.000000',
  `taker` decimal(7,6) NOT NULL DEFAULT '0.000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_trading_fees_on_market_id_and_market_type_and_group` (`market_id`,`market_type`,`group`),
  KEY `index_trading_fees_on_group` (`group`),
  KEY `index_trading_fees_on_market_id_and_market_type` (`market_id`,`market_type`)
) ENGINE=InnoDB AUTO_INCREMENT=6 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `trading_fees`
--

LOCK TABLES `trading_fees` WRITE;
/*!40000 ALTER TABLE `trading_fees` DISABLE KEYS */;
INSERT INTO `trading_fees` VALUES (1,'any','spot','any',0.002000,0.002000,'2021-05-20 14:43:58','2021-05-20 14:43:58'),(2,'any','spot','vip-0',0.001000,0.002000,'2021-05-20 14:43:58','2021-05-20 14:43:58'),(3,'any','spot','vip-1',0.000800,0.001800,'2021-05-20 14:43:58','2021-05-20 14:43:58'),(4,'any','spot','vip-2',0.000600,0.001600,'2021-05-20 14:43:58','2021-05-20 14:43:58'),(5,'any','spot','vip-3',0.000000,0.001400,'2021-05-20 14:43:58','2021-05-20 14:43:58');
/*!40000 ALTER TABLE `trading_fees` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `transactions`
--

DROP TABLE IF EXISTS `transactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `transactions` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `currency_id` varchar(255) NOT NULL,
  `reference_type` varchar(255) DEFAULT NULL,
  `reference_id` bigint(20) DEFAULT NULL,
  `txid` varchar(255) DEFAULT NULL,
  `from_address` varchar(255) DEFAULT NULL,
  `to_address` varchar(255) DEFAULT NULL,
  `amount` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `block_number` int(11) DEFAULT NULL,
  `txout` int(11) DEFAULT NULL,
  `status` varchar(255) DEFAULT NULL,
  `options` json DEFAULT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_transactions_on_currency_id_and_txid` (`currency_id`,`txid`),
  KEY `index_transactions_on_currency_id` (`currency_id`),
  KEY `index_transactions_on_reference_type_and_reference_id` (`reference_type`,`reference_id`),
  KEY `index_transactions_on_txid` (`txid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `transactions`
--

LOCK TABLES `transactions` WRITE;
/*!40000 ALTER TABLE `transactions` DISABLE KEYS */;
/*!40000 ALTER TABLE `transactions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `transfers`
--

DROP TABLE IF EXISTS `transfers`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `transfers` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `key` varchar(30) NOT NULL,
  `category` tinyint(4) NOT NULL,
  `description` varchar(255) DEFAULT '',
  `created_at` datetime(3) NOT NULL,
  `updated_at` datetime(3) NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_transfers_on_key` (`key`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `transfers`
--

LOCK TABLES `transfers` WRITE;
/*!40000 ALTER TABLE `transfers` DISABLE KEYS */;
/*!40000 ALTER TABLE `transfers` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `triggers`
--

DROP TABLE IF EXISTS `triggers`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `triggers` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `order_id` bigint(20) NOT NULL,
  `order_type` tinyint(3) unsigned NOT NULL,
  `value` varbinary(128) NOT NULL,
  `state` tinyint(3) unsigned NOT NULL DEFAULT '0',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_triggers_on_order_id` (`order_id`),
  KEY `index_triggers_on_order_type` (`order_type`),
  KEY `index_triggers_on_state` (`state`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `triggers`
--

LOCK TABLES `triggers` WRITE;
/*!40000 ALTER TABLE `triggers` DISABLE KEYS */;
/*!40000 ALTER TABLE `triggers` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `wallets`
--

DROP TABLE IF EXISTS `wallets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `wallets` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `blockchain_key` varchar(32) DEFAULT NULL,
  `name` varchar(64) DEFAULT NULL,
  `address` varchar(255) NOT NULL,
  `kind` int(11) NOT NULL,
  `gateway` varchar(20) NOT NULL DEFAULT '',
  `settings_encrypted` varchar(1024) DEFAULT NULL,
  `balance` json DEFAULT NULL,
  `max_balance` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `status` varchar(32) DEFAULT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `index_wallets_on_kind_and_currency_id_and_status` (`kind`,`status`),
  KEY `index_wallets_on_kind` (`kind`),
  KEY `index_wallets_on_status` (`status`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `wallets`
--

LOCK TABLES `wallets` WRITE;
/*!40000 ALTER TABLE `wallets` DISABLE KEYS */;
INSERT INTO `wallets` VALUES (1,'eth-rinkeby','Ethereum Deposit Wallet','0x2b9fBC10EbAeEc28a8Fc10069C0BC29E45eBEB9C',100,'geth','vault:dev:E3KOCmMWmuQGg6zbxJ9udVvloQreLug56SmjkiNkPYuSM88CP05im0qtS2KAtAHhpRFVNsJNczDcluFOIA39Ug==',NULL,0.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(2,'eth-rinkeby','Ethereum Hot Wallet','0x270704935783087a01c7a28d8f2d8f01670c8050',310,'geth','vault:dev:E3KOCmMWmuQGg6zbxJ9udVvloQreLug56SmjkiNkPYtVfUBuPMw+JeRa55XmUHyq',NULL,100.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(3,'eth-rinkeby','Ethereum Warm Wallet','0x2b9fBC10EbAeEc28a8Fc10069C0BC29E45eBEB9C',320,'geth','vault:dev:E3KOCmMWmuQGg6zbxJ9udVvloQreLug56SmjkiNkPYtVfUBuPMw+JeRa55XmUHyq',NULL,1000.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(4,'eth-rinkeby','Ethereum Wallet for paying ERC20 fees','0x270704935783087a01c7a28d8f2d8f01670c8050',200,'geth','vault:dev:E3KOCmMWmuQGg6zbxJ9udVvloQreLug56SmjkiNkPYtVfUBuPMw+JeRa55XmUHyq',NULL,100.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(5,'btc-testnet','Bitcoin Deposit Wallet','2N4qYjye5yENLEkz4UkLFxzPaxJatF3kRwf',100,'bitcoind','vault:dev:FqlV4tLyRQGlwmKKCqqxhfPoj8ayu8jIKMG1wMX1L6AvhRuZyoAS7ShWsOX1urbk',NULL,0.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(6,'btc-testnet','Bitcoin Hot Wallet','2N4qYjye5yENLEkz4UkLFxzPaxJatF3kRwf',310,'bitcoind','vault:dev:FqlV4tLyRQGlwmKKCqqxhfPoj8ayu8jIKMG1wMX1L6D58riL4e6zaLltk0+hP1OtDrBcA0BKeD0JriBod0I8yLM0uumFPq4TrWCLzsPp9po=',NULL,0.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15'),(7,'btc-testnet','Bitcoin Warm Wallet','2N4qYjye5yENLEkz4UkLFxzPaxJatF3kRwf',320,'bitcoind','vault:dev:FqlV4tLyRQGlwmKKCqqxhfPoj8ayu8jIKMG1wMX1L6AvhRuZyoAS7ShWsOX1urbk',NULL,0.0000000000000000,'active','2021-03-24 08:03:15','2021-03-24 08:03:15');
/*!40000 ALTER TABLE `wallets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `whitelisted_smart_contracts`
--

DROP TABLE IF EXISTS `whitelisted_smart_contracts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `whitelisted_smart_contracts` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `description` varchar(255) DEFAULT NULL,
  `address` varchar(255) NOT NULL,
  `state` varchar(30) NOT NULL,
  `blockchain_key` varchar(32) NOT NULL,
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_whitelisted_smart_contracts_on_address_and_blockchain_key` (`address`,`blockchain_key`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `whitelisted_smart_contracts`
--

LOCK TABLES `whitelisted_smart_contracts` WRITE;
/*!40000 ALTER TABLE `whitelisted_smart_contracts` DISABLE KEYS */;
INSERT INTO `whitelisted_smart_contracts` VALUES (1,NULL,'0x6c0b51971650d28821ce30b15b02b9826a20b129','active','eth-mainet','2021-03-24 08:03:15','2021-03-24 08:03:15'),(2,NULL,'0x1522900b6dafac587d499a862861c0869be6e428','active','eth-mainet','2021-03-24 08:03:15','2021-03-24 08:03:15'),(3,NULL,'0xbbd602bb278edff65cbc967b9b62095ad5be23a3','active','eth-rinkeby','2021-03-24 08:03:15','2021-03-24 08:03:15'),(4,NULL,'0xe3cb6897d83691a8eb8458140a1941ce1d6e6dac','active','eth-rinkeby','2021-03-24 08:03:15','2021-03-24 08:03:15');
/*!40000 ALTER TABLE `whitelisted_smart_contracts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `withdraw_limits`
--

DROP TABLE IF EXISTS `withdraw_limits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `withdraw_limits` (
  `id` bigint(20) NOT NULL AUTO_INCREMENT,
  `group` varchar(32) NOT NULL DEFAULT 'any',
  `kyc_level` varchar(32) NOT NULL DEFAULT 'any',
  `limit_24_hour` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `limit_1_month` decimal(32,16) NOT NULL DEFAULT '0.0000000000000000',
  `created_at` datetime NOT NULL,
  `updated_at` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_withdraw_limits_on_group_and_kyc_level` (`group`,`kyc_level`),
  KEY `index_withdraw_limits_on_group` (`group`),
  KEY `index_withdraw_limits_on_kyc_level` (`kyc_level`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `withdraw_limits`
--

LOCK TABLES `withdraw_limits` WRITE;
/*!40000 ALTER TABLE `withdraw_limits` DISABLE KEYS */;
/*!40000 ALTER TABLE `withdraw_limits` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `withdraws`
--

DROP TABLE IF EXISTS `withdraws`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `withdraws` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `member_id` int(11) NOT NULL,
  `beneficiary_id` bigint(20) DEFAULT NULL,
  `currency_id` varchar(10) NOT NULL,
  `amount` decimal(32,16) NOT NULL,
  `fee` decimal(32,16) NOT NULL,
  `txid` varchar(128) CHARACTER SET utf8 COLLATE utf8_bin DEFAULT NULL,
  `aasm_state` varchar(30) NOT NULL,
  `block_number` int(11) DEFAULT NULL,
  `sum` decimal(32,16) NOT NULL,
  `type` varchar(30) NOT NULL,
  `transfer_type` int(11) DEFAULT NULL,
  `tid` varchar(64) CHARACTER SET utf8 COLLATE utf8_bin NOT NULL,
  `rid` varchar(256) NOT NULL,
  `note` varchar(256) DEFAULT NULL,
  `metadata` json DEFAULT NULL,
  `error` json DEFAULT NULL,
  `created_at` datetime(3) NOT NULL,
  `updated_at` datetime(3) NOT NULL,
  `completed_at` datetime(3) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `index_withdraws_on_currency_id_and_txid` (`currency_id`,`txid`),
  KEY `index_withdraws_on_aasm_state` (`aasm_state`),
  KEY `index_withdraws_on_currency_id` (`currency_id`),
  KEY `index_withdraws_on_member_id` (`member_id`),
  KEY `index_withdraws_on_tid` (`tid`),
  KEY `index_withdraws_on_type` (`type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `withdraws`
--

LOCK TABLES `withdraws` WRITE;
/*!40000 ALTER TABLE `withdraws` DISABLE KEYS */;
/*!40000 ALTER TABLE `withdraws` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2021-05-21 11:40:08
