#![cfg_attr(not(test), no_main)]
extern crate alloc;
use openzeppelin_stylus::{access::ownable::{Ownable,Error as OwnableError}, token::erc20::{extensions::Erc20Metadata, Erc20}};
use alloy_sol_types::sol;
use stylus_sdk::{alloy_primitives::{Address, U256}, alloy_sol_types, prelude::{entrypoint, storage, public}, stylus_proc::SolidityError};

sol! {
    #[derive(Debug)]
    #[allow(missing_docs)]
    error MintInvalidReceiver(address receiver);

    #[derive(Debug)]
    #[allow(missing_docs)]
    error MintOperationError(address account, uint256 value);
}

#[derive(SolidityError, Debug)]
pub enum Error {
    InvalidReceiver(MintInvalidReceiver),
    MintError(MintOperationError),
    Ownable(OwnableError),
}

#[entrypoint]
#[storage]
struct Erc20Example {
    #[borrow]
    pub erc20: Erc20,
    #[borrow]
    pub metadata: Erc20Metadata,
    #[borrow]
    pub ownable: Ownable,
}

#[public]
#[inherit(Erc20,Erc20Metadata,Ownable)]
impl Erc20Example { 

    pub fn mint(
        &mut self,
        account: Address,
        value: U256,
    ) -> Result<(), Error> {
        self.ownable.only_owner()?;
        self.erc20._mint(account, value).map_err(|err| match err {
            openzeppelin_stylus::token::erc20::Error::InvalidReceiver(_)  => Error::InvalidReceiver(MintInvalidReceiver{receiver: account}),
            _ => Error::MintError(MintOperationError{account, value}),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use openzeppelin_stylus::token::erc20::IErc20;
    use stylus_sdk::{alloy_primitives::{address, uint, U256}, msg};
    use crate::{Error, Erc20Example};

    #[motsu::test]
    fn initial_balance_is_zero(contract: Erc20Example) {
        let test_address = address!("1234567891234567891234567891234567891234");
        let zero = U256::ZERO;
        
        let balance = contract.erc20.balance_of(test_address);
        assert_eq!(balance, zero);
    }

    #[motsu::test]
    fn owner_mints_tokens(contract: Erc20Example) {
        let test_address = address!("1234567891234567891234567891234567891234");
        let ten = uint!(10_U256);

        contract.ownable._owner.set(msg::sender());
        let _ = contract.mint(test_address,ten); 
        let balance = contract.erc20.balance_of(test_address);

        assert_eq!(balance, ten);
    }

    #[motsu::test]
    fn owner_mints_tokens_to_address_zero(contract: Erc20Example) {
        let test_address = address!("0000000000000000000000000000000000000000");
        let ten = uint!(10_U256);

        contract.ownable._owner.set(msg::sender());
        let result = contract.mint(test_address,ten); 

        assert!(matches!(result,Err(Error::InvalidReceiver(_))))
    }

    #[motsu::test]
    fn not_owner_tries_to_mint_tokens(contract: Erc20Example) {
        let test_address = address!("1234567891234567891234567891234567891234");
        let not_owner_address = address!("9123456789123456789123456789123456789123");
        let ten = uint!(10_U256);

        contract.ownable._owner.set(not_owner_address);
        let result = contract.mint(test_address,ten); 

        assert!(matches!(result,Err(Error::Ownable(_))))
    }
}