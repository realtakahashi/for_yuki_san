use crate::{
    traits::MultiAssetEvents,
    MultiAssetData,
};

use rmrk_common::{
    errors::{
        Result,
        RmrkError,
    },
    types::*,
    utils::Utils,
};

use ink::prelude::vec::Vec;

use openbrush::{
    contracts::psp34::extensions::enumerable::*,
    traits::Storage,
};
use ink::env::hash;

use core::{time::Duration};

/// Trait definitions for MultiAsset helper functions
pub trait Internal {
    /// Check if asset is already accepted. Return error if it is
    fn ensure_not_accepted(&self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    /// Check if asset is already pending. Return error if it is
    fn ensure_not_pending(&self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    /// Check if asset is already pending. Return OK if it is
    fn ensure_pending(&self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    /// Check if asset is already accepted
    fn ensure_asset_accepted(&self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    /// Add the asset to the list of accepted assets
    fn add_to_accepted_assets(&mut self, token_id: &Id, asset_id: &AssetId);

    /// Add the asset to the list of pending assets
    fn add_to_pending_assets(&mut self, token_id: &Id, asset_id: &AssetId);

    /// Replace asset by another AssetId
    fn replace_asset(
        &mut self,
        token_id: &Id,
        asset_id: &AssetId,
        replace_with_id: &AssetId,
    ) -> Result<()>;

    /// Remove the asset to the list of pending assets
    fn remove_from_pending_assets(&mut self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    /// Remove the asset to the list of accepted assets
    fn remove_from_accepted_assets(&mut self, token_id: &Id, asset_id: &AssetId) -> Result<()>;

    fn get_pseudo_random(&mut self, max_value: u8) -> u8;

    fn has_passed(&self, check_time: u64, last_time: u64) -> bool;

    fn five_minutes_has_passed(&self, last_time: u64) -> bool;

    fn one_day_has_passed(&self, last_time: u64) -> bool;

    
}

/// Implement internal helper trait for MultiAsset
impl<T> Internal for T
where
    T: Storage<MultiAssetData> + Storage<psp34::Data<enumerable::Balances>> + Utils,
{
    /// Check if asset is already accepted
    default fn ensure_not_accepted(&self, token_id: &Id, asset_id: &AssetId) -> Result<()> {
        if let Some(children) = self.data::<MultiAssetData>().accepted_assets.get(token_id) {
            if children.contains(asset_id) {
                return Err(RmrkError::AlreadyAddedAsset.into())
            }
        }
        Ok(())
    }

    /// Check if asset is already pending
    default fn ensure_not_pending(&self, token_id: &Id, asset_id: &AssetId) -> Result<()> {
        if let Some(assets) = self.data::<MultiAssetData>().pending_assets.get(token_id) {
            if assets.contains(asset_id) {
                return Err(RmrkError::AddingPendingAsset.into())
            }
        }
        Ok(())
    }

    /// Check if asset is already pending
    default fn ensure_pending(&self, token_id: &Id, asset_id: &AssetId) -> Result<()> {
        if let Some(assets) = self.data::<MultiAssetData>().pending_assets.get(token_id) {
            if !assets.contains(asset_id) {
                return Err(RmrkError::AssetIdNotFound.into())
            }
        }
        Ok(())
    }

    /// Check if asset is already accepted
    default fn ensure_asset_accepted(&self, token_id: &Id, asset_id: &AssetId) -> Result<()> {
        if let Some(assets) = self.data::<MultiAssetData>().accepted_assets.get(token_id) {
            if !assets.contains(asset_id) {
                return Err(RmrkError::AssetIdNotFound.into())
            }
        }
        Ok(())
    }

    /// Add the asset to the list of accepted assets
    default fn add_to_accepted_assets(&mut self, token_id: &Id, asset_id: &AssetId) {
        let mut assets = self
            .data::<MultiAssetData>()
            .accepted_assets
            .get(token_id)
            .unwrap_or(Vec::new());
        if !assets.contains(asset_id) {
            assets.push(*asset_id);
            self.data::<MultiAssetData>()
                .accepted_assets
                .insert(token_id, &assets);
        }
        self._emit_asset_accepted_event(token_id, asset_id);
    }

    /// Add the asset to the list of pending assets
    default fn add_to_pending_assets(&mut self, token_id: &Id, asset_id: &AssetId) {
        let mut assets = self
            .data::<MultiAssetData>()
            .pending_assets
            .get(token_id)
            .unwrap_or(Vec::new());
        if !assets.contains(asset_id) {
            assets.push(*asset_id);
            self.data::<MultiAssetData>()
                .pending_assets
                .insert(token_id, &assets);
        }
    }

    /// remove the asset from the list of pending assets
    default fn remove_from_pending_assets(
        &mut self,
        token_id: &Id,
        asset_id: &AssetId,
    ) -> Result<()> {
        let mut assets = self
            .data::<MultiAssetData>()
            .pending_assets
            .get(token_id)
            .ok_or(RmrkError::InvalidAssetId)?;

        let index = assets
            .iter()
            .position(|a| a == asset_id)
            .ok_or(RmrkError::InvalidTokenId)?;
        assets.remove(index);

        self.data::<MultiAssetData>()
            .pending_assets
            .insert(token_id, &assets);

        Ok(())
    }

    /// Remove the asset from the list of accepted assets
    default fn remove_from_accepted_assets(
        &mut self,
        token_id: &Id,
        asset_id: &AssetId,
    ) -> Result<()> {
        let mut assets = self
            .data::<MultiAssetData>()
            .accepted_assets
            .get(token_id)
            .ok_or(RmrkError::InvalidAssetId)?;

        let index = assets
            .iter()
            .position(|a| a == asset_id)
            .ok_or(RmrkError::InvalidTokenId)?;
        assets.remove(index);

        self.data::<MultiAssetData>()
            .accepted_assets
            .insert(token_id, &assets);

        Ok(())
    }

    // TODO:
    // * add replace pending storage ( ie collection issuer might suggest asset replace on a token with different owner )
    // * add "upsert" operation ( if replace failed add as a new asset )
    default fn replace_asset(
        &mut self,
        token_id: &Id,
        asset_id: &AssetId,
        replace_with_id: &AssetId,
    ) -> Result<()> {
        let mut accepted_list = self
            .data::<MultiAssetData>()
            .accepted_assets
            .get(token_id)
            .ok_or(RmrkError::AcceptedAssetsMissing)?;

        let asset_index = accepted_list
            .iter()
            .position(|x| x == replace_with_id)
            .ok_or(RmrkError::InvalidAssetId)?;

        accepted_list[asset_index] = *asset_id;
        self.data::<MultiAssetData>()
            .accepted_assets
            .insert(token_id, &accepted_list);

        Ok(())
    }

    default fn get_pseudo_random(&mut self, max_value: u8) -> u8 {
        let seed = Self::env().block_timestamp();
        let mut input: Vec<u8> = Vec::new();
        input.extend_from_slice(&seed.to_be_bytes());
        input.extend_from_slice(&self.data::<MultiAssetData>().salt.to_be_bytes());
        let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
        ink::env::hash_bytes::<hash::Keccak256>(&input, &mut output);
        self.data::<MultiAssetData>().salt += 1;
        let number = output[0] % (max_value + 1);
        number
    }

    default fn has_passed(&self, check_time :u64, last_time :u64) -> bool{
        let current_time = Self::env().block_timestamp();
        let time_since_last_time = current_time - last_time;
        let duration_time = Duration::from_secs(check_time);
        if Duration::from_millis(time_since_last_time) > duration_time {
            true
        } else {
            false
        }
    }

    default fn five_minutes_has_passed(&self, last_time :u64) -> bool{
        self.has_passed(60,last_time)
    }

    default fn one_day_has_passed(&self, last_time :u64) -> bool{
        self.has_passed(60 * 60 * 24 ,last_time)
    }
}
