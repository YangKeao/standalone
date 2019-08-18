# Standalone

This is a cargo subcommand `cargo standalone` which is used to check whether a mod depends on other mods and whether it can be extracted out as a standalone component.

## Usage

```
cargo standalone --entry src/lib.rs --mod storage::lock_manager
```

Run these command on tikv repo will lead print

```
storage::lock_manager::deadlock crate::server::resolve::StoreAddrResolver
storage::lock_manager::util crate::storage::mvcc::Error
storage::lock_manager::util crate::storage::txn
storage::lock_manager::util crate::storage::Error
storage::lock_manager::util crate::storage::Key
storage::lock_manager::waiter_manager crate::storage::mvcc::Error
storage::lock_manager::waiter_manager crate::storage::txn::Error
storage::lock_manager::waiter_manager crate::storage::txn
storage::lock_manager::waiter_manager crate::storage
storage::lock_manager::waiter_manager::tests crate::storage::Key
```

Every line contains a pair of string. The first part tells you which mod depends on mod outside mod. The second part tells you which mod it depends on.

## TODO

- Automatically extract a standalone mod.
- Calculate use path in tree but not path. To fix bug when facing `super::super::some::{super::super::some2}`.
