# Gensokyo - A Database Simulator

> :warning: it's still under design.

Gensokyo is a distributed transaction simulator, which allows you to implement your own database ideas. It’s hard and expensive to implement them in a complete system, you may suffer corner cases with non-related features, and many weird codes. In this framework, you just implement your algorithm, bench the performance and verify the correctness.

# Basic Conception

## Cluster

## Node

There are nodes in a cluster.

## Server

The server is a specific node in the cluster and should provide a KV transactional API for bench and test. It’s 

## Txn

Txn is the abbreviation of the transaction, the client sends txns to server(s). I conclude the forms of txns as the following:

* Basic KV operation vector
* Interactive transaction
* Stored procedures

## Request

The communication of nodes inside cluster.

## Storage

Storage data in either memory or disk or something else.
