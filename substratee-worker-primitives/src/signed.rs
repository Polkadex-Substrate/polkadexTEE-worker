use codec::{Decode, Encode};
use sp_core::crypto::{AccountId32, Pair};
use sp_core::ed25519;
use sp_runtime::{traits::Verify, MultiSignature};

pub type Signature = MultiSignature;

#[derive(Encode, Decode, Debug)]
pub struct SignedData<Data> {
    data: Data,
    signature: Signature,
}

impl<Data: Clone + Encode> SignedData<Data> {
    pub fn new(data: Data, pair: &ed25519::Pair) -> Self {
        let payload = data.encode();
        SignedData {
            data,
            signature: pair.sign(payload.as_slice()).into(),
        }
    }

    pub fn from(data: Data, signature: Signature) -> Self {
        SignedData { data, signature }
    }

    /// get block reference
    pub fn data(&self) -> &Data {
        &self.data
    }
    /// get signature reference
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Verifes the signature of a Block
    pub fn verify_signature(&self, signer: AccountId32) -> bool {
        // get block payload
        let payload = self.data.encode();

        // verify signature
        self.signature.verify(payload.as_slice(), &signer)
    }
}
