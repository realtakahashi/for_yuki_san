//! RMRK MultiAsset implementation
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
#![allow(clippy::inline_fn_without_body)]

pub mod internal;
pub mod traits;

use internal::Internal;

use ink::prelude::string::ToString;

use my_psp22_mintable::{ ContractRef};
use openbrush::traits::Balance;


use rmrk_common::{
    errors::{
        Result,
        RmrkError,
    },
    roles::CONTRIBUTOR,
    types::*,
    utils::Utils,
};

use traits::{
    MultiAsset,
    MultiAssetEvents,
};

use ink::{
    prelude::vec::Vec,
    storage::Mapping,
};

use openbrush::{
    contracts::{
        access_control::*,
        psp34::extensions::enumerable::*,
    },
    modifiers,
    traits::{
        AccountId,
        Storage,
        String,
    },
};

pub const STORAGE_MULTIASSET_KEY: u32 = openbrush::storage_unique_key!(MultiAssetData);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_MULTIASSET_KEY)]
pub struct MultiAssetData {
    /// Mapping of available asset entries for this collection
    pub collection_asset_entries: Mapping<AssetId, Asset>,

    /// Collection asset id list
    pub collection_asset_ids: Vec<AssetId>,

    /// Mapping of tokenId to an array of active assets
    pub accepted_assets: Mapping<Id, Vec<AssetId>>,

    /// Mapping of tokenId to an array of pending assets
    pub pending_assets: Mapping<Id, Vec<AssetId>>,

    /// Catalog assigned to assetId. Added with add_asset_entry
    /// An asset can also have None as a catalog, hence the Option
    pub asset_catalog_address: Mapping<AssetId, Option<AccountId>>,

    pub asset_status: Mapping<Id, Status>,

    // this is three pattern uri
    pub normal_uri: String,
    pub good_uri: String,
    pub bad_uri: String,

    // ランダム用
    pub salt: u64,

    // 前回食べた時間
    pub last_eaten: Mapping<Id, u64>,

    // 前回デイリーボーナスを取得した時間
    pub last_bonus: Mapping<AccountId, u64>,

    // 前回ステーキングした時間
    pub last_staked: Mapping<AccountId, u64>,

    // アカウントが保持しているリンゴの数
    pub apple_number: Mapping<AccountId, u16>,

    // アカウントが保持しているゲーム内通貨
    pub your_money: Mapping<AccountId, u64>,

    // ステーキングしているゲーム内通貨
    pub your_staked_money: Mapping<AccountId, u64>,

    
}

impl<T> MultiAsset for T
where
    T: Storage<MultiAssetData>
        + Storage<psp34::Data<enumerable::Balances>>
        + Storage<access_control::Data>
        + Utils,
{
    /// Used to add a asset entry.
    /// 
    /// 
    fn set_default(&mut self, account_id: AccountId) -> Result<()> {
        self.set_bad_uri(String::from("ipfs://QmYJhYes1kzp2soWYEYKzvA84V8YivL8BCpsnN773xyufr/"))?;
        self.set_normal_uri(String::from("ipfs://QmXtnr9aEJVywiLs1keZdyiKbQwignZT3FhwKYivF15oZp/"))?;
        self.set_good_uri(String::from("ipfs://QmZAdpKf4zr9x2vX26gU6LkG8gtj44GhoGMbWJAa2HsVzt/"))?;
        self.set_your_apple(account_id, 10)?;
        self.set_your_money(account_id, 500)?;
    Ok(())
}
    fn set_status (
        &mut self,
        token_id: Id, 
        hungry: u32,
        health: u32,
        happy: u32
    ) -> Result<()>{ 
        self.ensure_exists_and_get_owner(&token_id)?;
        self.data::<MultiAssetData>()
            .asset_status
            .insert(
                token_id,
                &Status {
                    hungry,
                    health,
                    happy,
                },
            );
        Ok(())
    }
    
    fn set_full_status(&mut self, token_id: Id) -> Result<()> {
        self.set_status(token_id, 0, 100, 100)
    }

    fn set_death_status(&mut self, token_id: Id) -> Result<()> {
        self.set_status(token_id, 80, 0, 0)
    }

    /// Used to retrieve Status
    fn get_status(&self, token_id: Id) -> Option<Status> {
        self.data::<MultiAssetData>()
            .asset_status
            .get(token_id)
    }

    fn get_current_status(&self, token_id: Id) -> Option<Status> {

        //　get the current time
        let current_time = Self::env().block_timestamp();

         // get the last eaten time
         let last_checked_time = self.data::<MultiAssetData>()
            .last_eaten
            .get(&token_id)
            .unwrap_or(Default::default());
        if last_checked_time == 0 {
            return Some(Status {
                hungry: 0,
                health: 0,
                happy: 0,
            });
        } else {
        
            let past_time = current_time - last_checked_time;

            // 60 seconds（60 ※ 1000 miliseconds）
            let past_day = past_time / (60 * 1000) ;
            // Assuming a hypothetical decrease of 5 per unit
            let change_status = past_day * 5;

            let original_status = self.get_status(token_id.clone()).unwrap_or_else(|| {
                // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
                Status { hungry: 0, health: 0, happy: 0 }
            });

            let new_hungy_status = original_status.hungry + (change_status as u32);
            let new_health_status = original_status.health.saturating_sub(change_status as u32);
            let new_happy_status = original_status.happy.saturating_sub(change_status as u32);

            return Some(Status {
                hungry: new_hungy_status,
                health: new_health_status,
                happy: new_happy_status,
            });
        }
    }

    // fn call_psp22(&mut self) {

    // }
    fn test_call_psp22(&mut self, target_account_id:AccountId, to: AccountId, value: Balance, data: Vec<u8>){
        let mut interface: ContractRef = ink::env::call::FromAccountId::from_account_id(target_account_id);
        interface.transfer_interface(to, value, data);

    }

    fn add_twenty(&mut self, token_id: Id) -> Result<()> {
        let original_status = self.get_current_status(token_id.clone()).unwrap_or_else(|| {
            // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
            Status { hungry: 0, health: 0, happy: 0 }
        });

        let hungry_status: u32;
        if original_status.hungry > 20 {
            hungry_status = original_status.hungry - 20;
        } else {
            hungry_status = 0;
        }
        let new_status = Status {
            hungry: hungry_status,
            health: original_status.health + 20,
            happy: original_status.happy + 20,
        };
    
        self.data::<MultiAssetData>()
            .asset_status
            .insert(token_id, &new_status);
        Ok(())
    }

    // to change some status
    fn change_some_status(&mut self, token_id: Id, number: u32) -> Result<()> {
        let original_status = self.get_current_status(token_id.clone()).unwrap_or_else(|| {
            // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
            Status { hungry: 0, health: 0, happy: 0 }
        });

        let hungry_status: u32;
        if original_status.hungry > number {
            hungry_status = original_status.hungry - number;
        } else {
            hungry_status = 0;
        }
    
        let new_status = Status {
            hungry: hungry_status,
            health: original_status.health + number,
            happy: original_status.happy + number,
        };
    
        self.data::<MultiAssetData>()
            .asset_status
            .insert(token_id, &new_status);
        Ok(())
    }

    fn set_lucky_status(&mut self, token_id: Id) -> Result<()> {
        self.change_some_status(token_id.clone(),50)
    }
    
    // 3 tyepes of uri function
    // 1) normal uri
    #[modifiers(only_role(CONTRIBUTOR))]
    fn set_normal_uri(&mut self, normal_uri:String) -> Result<()>{
        self.data::<MultiAssetData>()
        .normal_uri = normal_uri;
        Ok(())
    }

    fn get_normal_uri(&self) -> String {
        self.data::<MultiAssetData>()
            .normal_uri.clone()
    }

    // 2) good uri
    #[modifiers(only_role(CONTRIBUTOR))]
    fn set_good_uri(&mut self, good_uri:String) -> Result<()>{
        self.data::<MultiAssetData>()
        .good_uri = good_uri;
        Ok(())
    }

    
    fn get_good_uri(&self) -> String {
        self.data::<MultiAssetData>()
            .good_uri.clone()
    }

    // 3) bad uri
    #[modifiers(only_role(CONTRIBUTOR))]
    fn set_bad_uri(&mut self, bad_uri:String) -> Result<()>{
        self.data::<MultiAssetData>()
        .bad_uri = bad_uri;
        Ok(())
    }

    fn get_bad_uri(&self) -> String {
        self.data::<MultiAssetData>()
            .bad_uri.clone()
    }

    fn get_total_status(&self, token_id: Id) -> u32 {
        let original_status = self.get_current_status(token_id.clone()).unwrap_or_else(|| {
            // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
            Status { hungry: 0, health: 0, happy: 0 }
        });
    
        let new_status = Status {
            hungry: original_status.hungry,
            health: original_status.health,
            happy: original_status.happy,
        };

        let total_status = new_status.health as i32 + new_status.happy as i32 - new_status.hungry as i32;
        let result = if total_status > 0 { total_status } else { 0 };
        result as u32
    }

    fn get_condition(&self , token_id: Id) -> u32 {
        let condition = self.get_total_status(token_id);
        // bad condition
        if condition < 100 {
            0
        } 
        // normal condition
        else if condition < 200 {
            1
        } 
        // good condition
        else {
            2
        }
    }

    fn get_condition_url(&self , token_id: Id) -> String {
        let condition = self.get_condition(token_id);
        if condition == 0 {
            self.get_bad_uri()
        } else if condition == 1 {
            self.get_normal_uri()
        } else {
            self.get_good_uri()
        }
    }

    fn eat_an_apple(&mut self, token_id: Id, account_id: AccountId) -> Result<()> {

        // 前回のリンゴを食べた時間を取得。エラーの場合は、０を返す（todo 仮で設定）
        let last_eaten = self.get_last_eaten(token_id.clone());
        // 決められた時間が経過したかの関数
        let has_passed = self.five_minutes_has_passed(last_eaten);

        //  決められた時間が経過していない場合
        if has_passed ==false {
            Err(RmrkError::TimeHasNotPassed.into())
        } else {
            //　現在時刻取得 
            let current_time = Self::env().block_timestamp();
            //  last_eatenに現在時刻を入れる
            self.set_last_eaten(token_id.clone(), current_time)?;
            //  リンゴの数を減らす
            self.minus_your_apple(account_id)?;

            // 疑似乱数による分岐
            let random = self.get_pseudo_random(100);
            if random < 25 {
                self.change_some_status(token_id, 30)
            } else if random < 50 {
                self.set_full_status(token_id)
            } else if random < 75 {
                self.set_lucky_status(token_id)
            } else {
                self.set_death_status(token_id)
            } 
        }
        
    }


    fn token_uri(&self , token_id: Id) -> String {
        let id_string:ink::prelude::string::String = match token_id.clone() {
            Id::U8(u8) => {
                let tmp: u8 = u8;
                tmp.to_string()
            }
            Id::U16(u16) => {
                let tmp: u16 = u16;
                tmp.to_string()
            }
            Id::U32(u32) => {
                let tmp: u32 = u32;
                tmp.to_string()
            }
            Id::U64(u64) => {
                let tmp: u64 = u64;
                tmp.to_string()
            }
            Id::U128(u128) => {
                let tmp: u128 = u128;
                tmp.to_string()
            }
            // _ => "0".to_string()
            Id::Bytes(value) => ink::prelude::string::String::from_utf8(value.clone()).unwrap(),
        };

        let base_uri:String = self.get_condition_url(token_id.clone());
        let tmp_uri: ink::prelude::string::String = ink::prelude::string::String::from_utf8(base_uri).unwrap();
        let uri:ink::prelude::string::String = tmp_uri + &id_string;

        uri.into_bytes()
    }

    fn get_your_apple(&self, account_id: AccountId) -> u16 {
        self.data::<MultiAssetData>()
            .apple_number
            .get(&account_id)
            .unwrap_or_default()
    }

    fn set_your_apple(&mut self, account_id: AccountId, after_apple: u16) -> Result<()> {
        self.data::<MultiAssetData>()
            .apple_number
            .insert(account_id, &after_apple);
        Ok(())
    }


    fn get_your_money(&self, account_id: AccountId) -> u64 {
        self.data::<MultiAssetData>()
            .your_money
            .get(&account_id)
            .unwrap_or_default()
    }

    fn set_your_money(&mut self, account_id: AccountId, after_money: u64) -> Result<()> {
        self.data::<MultiAssetData>()
            .your_money
            .insert(account_id, &after_money);
        Ok(())
    }

    fn stake_your_money(&mut self, account_id: AccountId, stake_money: u64) -> Result<()> {

        //　get the current time
        let current_time = Self::env().block_timestamp();

        //　get the current money
        let current_money = self.get_your_money(account_id.clone());

        //　get the current staked money
        let current_staked_money = self.get_your_staked_money(account_id.clone());

        if current_money == 0 || current_money < stake_money {
            Err(RmrkError::NotEnoughMoney.into())
        } else {
            let after_money = current_money - stake_money;

            let after_staked_money = current_staked_money + stake_money;
            // set your_money 0
            self.data::<MultiAssetData>()
                .your_money
                .insert(account_id, &after_money);

            // set your_staked_money
            self.data::<MultiAssetData>()
                .your_staked_money
                .insert(account_id, &after_staked_money);

            // set last_staked
            self.data::<MultiAssetData>()
                .last_staked
                .insert(account_id, &current_time);
            Ok(())
        }
    }

    fn get_your_staked_money(&self, account_id: AccountId) -> u64 {

        //　get the current time
        let current_time = Self::env().block_timestamp();

        // get your_staked_money
        let staked_money = self.data::<MultiAssetData>()
            .your_staked_money
            .get(&account_id)
            .unwrap_or(Default::default());

        // get last_staked_time
        let last_staked_time = self.data::<MultiAssetData>()
            .last_staked
            .get(&account_id)
            .unwrap_or(Default::default());
        if last_staked_time == 0 || staked_money == 0 {
            return 0
        } else {
            let past_time = current_time - last_staked_time;
            // 60 seconds（60 ※ 1000 miliseconds）
            let past_day = past_time / (10 * 1000) ;
            // Assuming a hypothetical decrease of 5 per unit
            let change_patio = past_day * 1;
            return staked_money + staked_money * change_patio / 100
        }
    }

    fn withdraw_your_money(&mut self, account_id: AccountId) -> Result<()> {
        let staked_money = self.get_your_staked_money(account_id);

        let current_money = self.get_your_money(account_id.clone());

        if staked_money == 0 {
            Err(RmrkError::NotEnoughMoney.into())
        } else {
            let result_money = current_money + staked_money;
            // set your_staked_money 0
            self.data::<MultiAssetData>()
            .your_staked_money
            .insert(account_id, &0);

            // set your_money 
            self.data::<MultiAssetData>()
                .your_money
                .insert(account_id, &result_money);
            Ok(())
        }
    }

    fn buy_an_apple(&mut self, account_id: AccountId) -> Result<()>{

        // 仮にリンゴの値段を20とする。エラーの場合は?があるため返る
        self.minus_your_money(account_id, 20)?;

        // １個加える
        let after_apple = self.get_your_apple(account_id) + 1;

        self.data::<MultiAssetData>()
            .apple_number
            .insert(account_id, &after_apple);

        Ok(())

    }

    fn minus_your_apple(&mut self, account_id: AccountId) -> Result<()> {
        
        // リンゴの数を取得する
        let apple_number = self.get_your_apple(account_id);

        if apple_number < 1 {
            Err(RmrkError::NotEnoughApple.into())
        } else {
            let after_apple = apple_number - 1;

            self.data::<MultiAssetData>()
            .apple_number
            .insert(account_id, &after_apple);
            Ok(())
        }
    }

    fn minus_your_money(&mut self, account_id: AccountId, change_money: u64) -> Result<()> {
        
        // 現在の所有のゲーム内通貨を取得する
        let money = self.get_your_money(account_id);

        if money < change_money {
            Err(RmrkError::NotEnoughMoney.into())
        } else {
            let after_money = money - change_money;
            self.set_your_money(account_id, after_money)?;
            Ok(())
        }
    }

    fn plus_your_money(&mut self, account_id: AccountId, change_money: u64) -> Result<()> {
        
        // 現在の所有のゲーム内通貨を取得する
        let money = self.get_your_money(account_id);

        let after_money = money + change_money;
        self.set_your_money(account_id, after_money)?;
        Ok(())
    }

    // 
    fn daily_bonus(&mut self, account_id: AccountId) -> Result<()> {

        let is_account_id = self.is_account_id(account_id);

        if is_account_id == false {
            Err(RmrkError::InvalidAccountId.into())
        } else {
            // Get the time when the last bonus was obtained. In case of error, return 0 
            let last_bonus = self.get_last_bonus(account_id);
            // Function of whether a predetermined amount of time has elapsed.
            let has_passed = self.five_minutes_has_passed(last_bonus);

            //  If the allotted time has not elapsed
            if has_passed ==false {
                Err(RmrkError::TimeHasNotPassed.into())
            } else {
            //　Get the current time
            let current_time = Self::env().block_timestamp();
            //  Put current time in last_bonus
            self.set_last_bonus(account_id, current_time)?;

            let after_money = self.get_your_money(account_id) + 100;
            self.set_your_money(account_id, after_money)?;

            Ok(())
            }
        
        }
    }

    fn get_last_eaten(&self, token_id: Id) -> u64 {
        self.data::<MultiAssetData>()
            .last_eaten
            .get(&token_id)
            .unwrap_or(Default::default())
    }

    fn set_last_eaten(&mut self, token_id: Id, current_time: u64) -> Result<()> {
        self.data::<MultiAssetData>()
            .last_eaten
            .insert(token_id, &current_time);
        Ok(())
    }

    fn get_last_bonus(&self, account_id: AccountId) -> u64 {
        self.data::<MultiAssetData>()
            .last_bonus
            .get(&account_id)
            .unwrap_or(Default::default())
    }

    fn set_last_bonus(&mut self, account_id: AccountId, current_time: u64) -> Result<()> {
        self.data::<MultiAssetData>()
            .last_bonus
            .insert(account_id, &current_time);
        Ok(())
    }

    fn is_nft_owner(&self, token_id: Id) -> bool {
        let token_owner = self
            .data::<psp34::Data<enumerable::Balances>>()
            .owner_of(token_id.clone())
            .unwrap();

        if token_owner == Self::env().caller() {
            true
        } else {
            false
        }
    }

    fn is_account_id(&self, account_id: AccountId) -> bool {
        let caller = Self::env().caller();
        if caller == account_id {
            true
        } else {
            false
        }
    }

    //  Used to add a asset entry.
    #[modifiers(only_role(CONTRIBUTOR))]
    fn add_asset_entry(
        &mut self,
        catalog_address: Option<AccountId>,
        asset_id: AssetId,
        equippable_group_id: EquippableGroupId,
        asset_uri: String,
        part_ids: Vec<PartId>,
    ) -> Result<()> {
        self.ensure_asset_id_is_available(asset_id)?;
        self.data::<MultiAssetData>()
            .collection_asset_entries
            .insert(
                asset_id,
                &Asset {
                    equippable_group_id,
                    asset_uri,
                    part_ids: part_ids.clone(),
                },
            );
        self.data::<MultiAssetData>()
            .collection_asset_ids
            .push(asset_id);
        self.data::<MultiAssetData>()
            .asset_catalog_address
            .insert(asset_id, &catalog_address);
        self._emit_asset_set_event(&asset_id);

        Ok(())
    }

    /// Used to add an asset to a token.
    /// tokenId - ID of the token to add the asset to
    /// assetId - ID of the asset to add to the token
    /// replacesAssetWithId - ID of the asset to replace from the token's list of active assets
    fn add_asset_to_token(
        &mut self,
        token_id: Id,
        asset_id: AssetId,
        replaces_asset_with_id: Option<AssetId>,
    ) -> Result<()> {
        // Check if asset id is valid
        self.data::<MultiAssetData>()
            .collection_asset_entries
            .get(asset_id)
            .ok_or(RmrkError::AssetIdNotFound)?;
        let token_owner = self.ensure_exists_and_get_owner(&token_id)?;
        self.ensure_not_accepted(&token_id, &asset_id)?;
        self.ensure_not_pending(&token_id, &asset_id)?;
        self._emit_asset_added_to_token_event(&token_id, &asset_id, &replaces_asset_with_id);

        if let Some(replace_with_id) = replaces_asset_with_id {
            ink::env::debug_println!("replaces_asset_with_id {:?}", &replaces_asset_with_id);
            return self.replace_asset(&token_id, &asset_id, &replace_with_id)
        } else {
            let caller = Self::env().caller();
            // If the asset is being added by the current root owner of the token, the asset will be automatically accepted.
            if caller == token_owner {
                self.add_to_accepted_assets(&token_id, &asset_id);
            } else {
                self.add_to_pending_assets(&token_id, &asset_id);
            }
        }

        Ok(())
    }

    /// Accepts an asset from the pending array of given token.
    fn accept_asset(&mut self, token_id: Id, asset_id: AssetId) -> Result<()> {
        self.ensure_pending(&token_id, &asset_id)?;
        let token_owner = self.ensure_exists_and_get_owner(&token_id)?;
        let caller = Self::env().caller();
        if caller == token_owner {
            self.remove_from_pending_assets(&token_id, &asset_id)?;
            self.add_to_accepted_assets(&token_id, &asset_id);
        } else {
            return Err(RmrkError::NotTokenOwner.into())
        }
        Ok(())
    }

    /// Rejects an asset from the pending array of given token.
    fn reject_asset(&mut self, token_id: Id, asset_id: AssetId) -> Result<()> {
        self.ensure_pending(&token_id, &asset_id)?;
        let token_owner = self.ensure_exists_and_get_owner(&token_id)?;
        self.ensure_token_owner(token_owner)?;

        self.remove_from_pending_assets(&token_id, &asset_id)?;

        self._emit_asset_rejected_event(&token_id, &asset_id);
        Ok(())
    }

    /// Remove an asset from the pending array of given token.
    fn remove_asset(&mut self, token_id: Id, asset_id: AssetId) -> Result<()> {
        self.ensure_asset_accepted(&token_id, &asset_id)?;
        let token_owner = self.ensure_exists_and_get_owner(&token_id)?;
        self.ensure_token_owner(token_owner)?;

        self.remove_from_accepted_assets(&token_id, &asset_id)?;

        self._emit_asset_removed_event(&token_id, &asset_id);
        Ok(())
    }

    /// Used to specify the priorities for a given token's active assets.
    fn set_priority(&mut self, token_id: Id, priorities: Vec<AssetId>) -> Result<()> {
        let token_owner = self.ensure_exists_and_get_owner(&token_id)?;
        self.ensure_token_owner(token_owner)?;
        if let Some(accepted_assets) = self
            .data::<MultiAssetData>()
            .accepted_assets
            .get(token_id.clone())
        {
            if accepted_assets.len() != priorities.len() {
                return Err(RmrkError::BadPriorityLength.into())
            }
            for asset in priorities.clone() {
                if !accepted_assets.contains(&asset) {
                    return Err(RmrkError::AssetIdNotFound.into())
                }
            }
        }

        self.data::<MultiAssetData>()
            .accepted_assets
            .insert(&token_id, &priorities);
        self._emit_asset_priority_set_event(&token_id, priorities);
        Ok(())
    }

    /// Used to retrieve the total number of asset entries
    fn total_assets(&self) -> u32 {
        self.data::<MultiAssetData>().collection_asset_ids.len() as u32
    }

    /// Used to retrieve the total number of assets per token
    fn total_token_assets(&self, token_id: Id) -> Result<(u64, u64)> {
        self.ensure_exists_and_get_owner(&token_id)?;

        let accepted_assets_on_token =
            match self.data::<MultiAssetData>().accepted_assets.get(&token_id) {
                Some(assets) => assets.len() as u64,
                None => 0,
            };

        let pending_assets_on_token =
            match self.data::<MultiAssetData>().pending_assets.get(&token_id) {
                Some(assets) => assets.len() as u64,
                None => 0,
            };

        Ok((accepted_assets_on_token, pending_assets_on_token))
    }

    /// Check that asset id does not already exist.
    default fn ensure_asset_id_is_available(&self, asset_id: AssetId) -> Result<()> {
        if self
            .data::<MultiAssetData>()
            .collection_asset_entries
            .get(asset_id)
            .is_some()
        {
            return Err(RmrkError::AssetIdAlreadyExists.into())
        }

        Ok(())
    }

    /// Used to retrieve asset's uri
    default fn get_asset_uri(&self, asset_id: AssetId) -> Option<String> {
        self.get_asset(asset_id).map(|asset| asset.asset_uri)
    }

    /// Used to retrieve asset
    default fn get_asset(&self, asset_id: AssetId) -> Option<Asset> {
        self.data::<MultiAssetData>()
            .collection_asset_entries
            .get(asset_id)
    }

    /// Fetch all accepted assets for the token_id
    fn get_accepted_token_assets(&self, token_id: Id) -> Result<Vec<AssetId>> {
        self.ensure_exists_and_get_owner(&token_id)?;
        Ok(self
            .data::<MultiAssetData>()
            .accepted_assets
            .get(&token_id)
            .unwrap_or_default())
    }

    /// Fetch all pending assets for the token_id
    fn get_pending_token_assets(&self, token_id: Id) -> Result<Vec<AssetId>> {
        self.ensure_exists_and_get_owner(&token_id)?;
        Ok(self
            .data::<MultiAssetData>()
            .pending_assets
            .get(&token_id)
            .unwrap_or_default())
    }

    /// Fetch asset's catalog
    fn get_asset_catalog_address(&self, asset_id: AssetId) -> Option<AccountId> {
        self.data::<MultiAssetData>()
            .asset_catalog_address
            .get(asset_id)
            .unwrap_or_default()
    }
}

/// Event trait for MultiAssets
impl<T> MultiAssetEvents for T
where
    T: Storage<MultiAssetData>,
{
    /// Used to notify listeners that an asset object is initialized at `assetId`.
    default fn _emit_asset_set_event(&self, _asset_id: &AssetId) {}

    /// Used to notify listeners that an asset object at `assetId` is added to token's pending asset array.
    default fn _emit_asset_added_to_token_event(
        &self,
        _token_id: &Id,
        _asset_id: &AssetId,
        _replaces_id: &Option<AssetId>,
    ) {
    }

    /// Used to notify listeners that an asset object at `assetId` is accepted by the token and migrated
    default fn _emit_asset_accepted_event(&self, _token_id: &Id, _asset_id: &AssetId) {}

    /// Used to notify listeners that an asset object at `assetId` is rejected from token and is dropped from the pending assets array of the token.
    default fn _emit_asset_rejected_event(&self, _token_id: &Id, _asset_id: &AssetId) {}

    /// Used to notify listeners that an asset object at `assetId` is removed from token
    default fn _emit_asset_removed_event(&self, _token_id: &Id, _asset_id: &AssetId) {}

    /// Used to notify listeners that token's prioritiy array is reordered.
    default fn _emit_asset_priority_set_event(&self, _token_id: &Id, _priorities: Vec<AssetId>) {}
}
