use {
    crate::{
        impl_deserialize_for_hex_string_wrapper,
        store::types::{
            PriceFeedUpdate,
            Slot,
            UnixTimestamp,
        },
    },
    base64::{
        engine::general_purpose::STANDARD as base64_standard_engine,
        Engine as _,
    },
    derive_more::{
        Deref,
        DerefMut,
    },
    pyth_sdk::{
        Price,
        PriceIdentifier,
    },
    wormhole_sdk::Chain,
};


/// PriceIdInput is a wrapper around a 32-byte hex string.
/// that supports a flexible deserialization from a hex string.
/// It supports both 0x-prefixed and non-prefixed hex strings,
/// and also supports both lower and upper case characters.
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct PriceIdInput([u8; 32]);
// TODO: Use const generics instead of macro.
impl_deserialize_for_hex_string_wrapper!(PriceIdInput, 32);

impl From<PriceIdInput> for PriceIdentifier {
    fn from(id: PriceIdInput) -> Self {
        Self::new(*id)
    }
}

type Base64String = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RpcPriceFeedMetadata {
    pub slot:                       Slot,
    pub emitter_chain:              u16,
    pub price_service_receive_time: UnixTimestamp,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RpcPriceFeed {
    pub id:        PriceIdentifier,
    pub price:     Price,
    pub ema_price: Price,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata:  Option<RpcPriceFeedMetadata>,
    /// Vaa binary represented in base64.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaa:       Option<Base64String>,
}

impl RpcPriceFeed {
    // TODO: Use a Encoding type to have None, Base64, and Hex variants instead of binary flag.
    // TODO: Use a Verbosity type to define None, or Full instead of verbose flag.
    pub fn from_price_feed_update(
        price_feed_update: PriceFeedUpdate,
        verbose: bool,
        binary: bool,
    ) -> Self {
        let price_feed_message = price_feed_update.price_feed;

        Self {
            id:        PriceIdentifier::new(price_feed_message.feed_id),
            price:     Price {
                price:        price_feed_message.price,
                conf:         price_feed_message.conf,
                expo:         price_feed_message.exponent,
                publish_time: price_feed_message.publish_time,
            },
            ema_price: Price {
                price:        price_feed_message.ema_price,
                conf:         price_feed_message.ema_conf,
                expo:         price_feed_message.exponent,
                publish_time: price_feed_message.publish_time,
            },
            metadata:  verbose.then_some(RpcPriceFeedMetadata {
                emitter_chain:              Chain::Pythnet.into(),
                price_service_receive_time: price_feed_update.received_at,
                slot:                       price_feed_update.slot,
            }),
            vaa:       binary.then_some(
                base64_standard_engine.encode(price_feed_update.wormhole_merkle_update_data),
            ),
        }
    }
}
