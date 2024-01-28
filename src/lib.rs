// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::{
    alloy_primitives::{
        Address,
        FixedBytes,
    },
    alloy_sol_types::{
        sol,
        SolError,
    },
    evm,
    msg,
    prelude::*,
};

sol! {
    error AlreadyInitialized();
    error NotPinger();

    event Ping();
    event Pong(bytes32 txHash);
}

sol_storage! {
    #[entrypoint]
    pub struct Contract {
        bool initialized;
        address pinger;
    }
}

pub enum ContractError {
    AlreadyInitialized(AlreadyInitialized),
    NotPinger(NotPinger),
}

impl From<ContractError> for Vec<u8> {
    fn from(val: ContractError) -> Self {
        match val {
            ContractError::NotPinger(err) => err.encode(),
            ContractError::AlreadyInitialized(err) => err.encode(),
        }
    }
}

type Result<T, E = ContractError> = core::result::Result<T, E>;

#[external]
impl Contract {
    pub fn init(&mut self) -> Result<()> {
        if self.initialized.get() {
            return Err(ContractError::AlreadyInitialized(AlreadyInitialized {}));
        }

        self.initialized.set(true);
        self.pinger.set(msg::sender());

        Ok(())
    }

    pub fn pinger(&self) -> Result<Address, Vec<u8>> {
        Ok(self.pinger.get())
    }

    pub fn ping(&self) -> Result<()> {
        if self.pinger.get() != msg::sender() {
            return Err(ContractError::NotPinger(NotPinger {}));
        }

        evm::log(
            Ping {}
        );

        Ok(())
    }

    pub fn pong(tx_hash: FixedBytes<32>) -> Result<(), Vec<u8>> {
        evm::log(
            Pong {
                txHash: *tx_hash,
            }
        );

        Ok(())
    }
}
