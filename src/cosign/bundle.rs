//
// Copyright 2021 The Sigstore Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use olpc_cjson::CanonicalFormatter;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

use crate::crypto::{CosignVerificationKey, Signature};
use crate::errors::{Result, SigstoreError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignedArtifactBundle {
    pub base64_signature: String,
    pub cert: String,
    pub rekor_bundle: Bundle,
}

impl SignedArtifactBundle {
    /// Create a new verified `SignedArtifactBundle`.
    ///
    /// **Note well:** The bundle will be returned only if it can be verified
    /// using the supplied `rekor_pub_key` public key.
    #[allow(dead_code)]
    pub(crate) fn new_verified(raw: &str, rekor_pub_key: &CosignVerificationKey) -> Result<Self> {
        let bundle: SignedArtifactBundle = serde_json::from_str(raw).map_err(|e| {
            SigstoreError::UnexpectedError(format!("Cannot parse bundle |{}|: {:?}", raw, e))
        })?;
        Bundle::verify_bundle(&bundle.rekor_bundle, rekor_pub_key).map(|_| bundle)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Bundle {
    pub signed_entry_timestamp: String,
    pub payload: Payload,
}

impl Bundle {
    /// Create a new verified `Bundle`
    ///
    /// **Note well:** The bundle will be returned only if it can be verified
    /// using the supplied `rekor_pub_key` public key.
    pub(crate) fn new_verified(raw: &str, rekor_pub_key: &CosignVerificationKey) -> Result<Self> {
        let bundle: Bundle = serde_json::from_str(raw).map_err(|e| {
            SigstoreError::UnexpectedError(format!("Cannot parse bundle |{}|: {:?}", raw, e))
        })?;
        Self::verify_bundle(&bundle, rekor_pub_key).map(|_| bundle)
    }

    /// Verify a `Bundle`.
    ///
    /// **Note well:** The bundle will be returned only if it can be verified
    /// using the supplied `rekor_pub_key` public key.
    pub(crate) fn verify_bundle(
        bundle: &Bundle,
        rekor_pub_key: &CosignVerificationKey,
    ) -> Result<()> {
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
        bundle.payload.serialize(&mut ser).map_err(|e| {
            SigstoreError::UnexpectedError(format!(
                "Cannot create canonical JSON representation of bundle: {:?}",
                e
            ))
        })?;

        rekor_pub_key.verify_signature(
            Signature::Base64Encoded(bundle.signed_entry_timestamp.as_bytes()),
            &buf,
        )?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    pub body: String,
    pub integrated_time: i64,
    pub log_index: i64,
    #[serde(rename = "logID")]
    pub log_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    use crate::cosign::tests::get_rekor_public_key;
    use crate::crypto::SigningScheme;

    fn build_correct_bundle() -> String {
        let bundle_json = json!({
          "SignedEntryTimestamp": "MEUCIDx9M+yRpD0O47/Mzm8NAPCbtqy4uiTkLWWexW0bo4jZAiEA1wwueIW8XzJWNkut5y9snYj7UOfbMmUXp7fH3CzJmWg=",
          "Payload": {
            "body": "eyJhcGlWZXJzaW9uIjoiMC4wLjEiLCJraW5kIjoicmVrb3JkIiwic3BlYyI6eyJkYXRhIjp7Imhhc2giOnsiYWxnb3JpdGhtIjoic2hhMjU2IiwidmFsdWUiOiIzYWY0NDE0ZDIwYzllMWNiNzZjY2M3MmFhZThiMjQyMTY2ZGFiZTZhZjUzMWE0YTc5MGRiOGUyZjBlNWVlN2M5In19LCJzaWduYXR1cmUiOnsiY29udGVudCI6Ik1FWUNJUURXV3hQUWEzWEZVc1BieVRZK24rYlp1LzZQd2hnNVd3eVlEUXRFZlFobzl3SWhBUGtLVzdldWI4YjdCWCtZYmJSYWM4VHd3SXJLNUt4dmR0UTZOdW9EK2l2VyIsImZvcm1hdCI6Ing1MDkiLCJwdWJsaWNLZXkiOnsiY29udGVudCI6IkxTMHRMUzFDUlVkSlRpQlFWVUpNU1VNZ1MwVlpMUzB0TFMwS1RVWnJkMFYzV1VoTGIxcEplbW93UTBGUldVbExiMXBKZW1vd1JFRlJZMFJSWjBGRlRFdG9SRGRHTlU5TGVUYzNXalU0TWxrMmFEQjFNVW96UjA1Qkt3cHJkbFZ6YURSbFMzQmtNV3gzYTBSQmVtWkdSSE0zZVZoRlJYaHpSV3RRVUhWcFVVcENaV3hFVkRZNGJqZFFSRWxYUWk5UlJWazNiWEpCUFQwS0xTMHRMUzFGVGtRZ1VGVkNURWxESUV0RldTMHRMUzB0Q2c9PSJ9fX19",
            "integratedTime": 1634714179,
            "logIndex": 783606,
            "logID": "c0d23d6ad406973f9559f3ba2d1ca01f84147d8ffc5b8445c224f98b9591801d"
          }
        });
        serde_json::to_string(&bundle_json).unwrap()
    }

    #[test]
    fn bundle_new_verified_success() {
        let rekor_pub_key = get_rekor_public_key();

        let bundle_json = build_correct_bundle();
        let bundle = Bundle::new_verified(&bundle_json, &rekor_pub_key);

        assert!(bundle.is_ok());
    }

    #[test]
    fn bundle_new_verified_failure() {
        let public_key = r#"-----BEGIN PUBLIC KEY-----
MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAENptdY/l3nB0yqkXLBWkZWQwo6+cu
OSWS1X9vPavpiQOoTTGC0xX57OojUadxF1cdQmrsiReWg2Wn4FneJfa8xw==
-----END PUBLIC KEY-----"#;
        let not_rekor_pub_key =
            CosignVerificationKey::from_pem(public_key.as_bytes(), &SigningScheme::default())
                .expect("Cannot create CosignVerificationKey");

        let bundle_json = build_correct_bundle();
        let bundle = Bundle::new_verified(&bundle_json, &not_rekor_pub_key);

        assert!(bundle.is_err());
    }

    #[test]
    fn signedartifactbundle_new_verified_success() {
        // Bundle as generated by running the following command, and taking the
        // content from the generated 'artifact.bundle` file:
        // cosign sign-blob --bundle=artifact.bundle artifact.txt
        let bundle_raw = r#"
{"base64Signature":"MEQCIGp1XZP5zaImosrBhDPCdXn3f8xI9FHGLsGVx6UeRPCgAiAt5GrsdQhOKnZcA3EWecvgJSHzCIjWifFBQkD7Hdsymg==","cert":"LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUNxRENDQWkrZ0F3SUJBZ0lVVFBXVGZPLzFOUmFTRmRlY2FBUS9wQkRHSnA4d0NnWUlLb1pJemowRUF3TXcKTnpFVk1CTUdBMVVFQ2hNTWMybG5jM1J2Y21VdVpHVjJNUjR3SEFZRFZRUURFeFZ6YVdkemRHOXlaUzFwYm5SbApjbTFsWkdsaGRHVXdIaGNOTWpJeE1USTFNRGN6TnpFeVdoY05Nakl4TVRJMU1EYzBOekV5V2pBQU1Ga3dFd1lICktvWkl6ajBDQVFZSUtvWkl6ajBEQVFjRFFnQUVKUVE0Vy81WFA5bTRZYldSQlF0SEdXd245dVVoYWUzOFVwY0oKcEVNM0RPczR6VzRNSXJNZlc0V1FEMGZ3cDhQVVVSRFh2UTM5NHBvcWdHRW1Ta3J1THFPQ0FVNHdnZ0ZLTUE0RwpBMVVkRHdFQi93UUVBd0lIZ0RBVEJnTlZIU1VFRERBS0JnZ3JCZ0VGQlFjREF6QWRCZ05WSFE0RUZnUVVvM0tuCmpKUVowWGZpZ2JENWIwT1ZOTjB4cVNvd0h3WURWUjBqQkJnd0ZvQVUzOVBwejFZa0VaYjVxTmpwS0ZXaXhpNFkKWkQ4d0p3WURWUjBSQVFIL0JCMHdHNEVaWkdGdWFXVnNMbUpsZG1WdWFYVnpRR2R0WVdsc0xtTnZiVEFzQmdvcgpCZ0VFQVlPL01BRUJCQjVvZEhSd2N6b3ZMMmRwZEdoMVlpNWpiMjB2Ykc5bmFXNHZiMkYxZEdnd2dZc0dDaXNHCkFRUUIxbmtDQkFJRWZRUjdBSGtBZHdEZFBUQnF4c2NSTW1NWkhoeVpaemNDb2twZXVONDhyZitIaW5LQUx5bnUKamdBQUFZU3R1Qkh5QUFBRUF3QklNRVlDSVFETTVZU1EvR0w2S0k1UjlPZGNuL3BTaytxVkQ2YnNMODMrRXA5UgoyaFdUYXdJaEFLMWppMWxaNTZEc2Z1TGZYN2JCQzluYlIzRWx4YWxCaHYxelFYTVU3dGx3TUFvR0NDcUdTTTQ5CkJBTURBMmNBTUdRQ01CSzh0c2dIZWd1aCtZaGVsM1BpakhRbHlKMVE1SzY0cDB4cURkbzdXNGZ4Zm9BUzl4clAKczJQS1FjZG9EOWJYd2dJd1g2ekxqeWJaa05IUDV4dEJwN3ZLMkZZZVp0ME9XTFJsVWxsY1VETDNULzdKUWZ3YwpHU3E2dlZCTndKMDB3OUhSCi0tLS0tRU5EIENFUlRJRklDQVRFLS0tLS0K","rekorBundle":{"SignedEntryTimestamp":"MEUCIC3c+21v9pk6o4BpB/dRAM9lGnyWLi3Xnc+i8LmnNJmeAiEAiqZJbZHx3Idnw+zXv6yM0ipPw/p16R28YGuCJFQ1u8U=","Payload":{"body":"eyJhcGlWZXJzaW9uIjoiMC4wLjEiLCJraW5kIjoiaGFzaGVkcmVrb3JkIiwic3BlYyI6eyJkYXRhIjp7Imhhc2giOnsiYWxnb3JpdGhtIjoic2hhMjU2IiwidmFsdWUiOiI0YmM0NTNiNTNjYjNkOTE0YjQ1ZjRiMjUwMjk0MjM2YWRiYTJjMGUwOWZmNmYwMzc5Mzk0OWU3ZTM5ZmQ0Y2MxIn19LCJzaWduYXR1cmUiOnsiY29udGVudCI6Ik1FUUNJR3AxWFpQNXphSW1vc3JCaERQQ2RYbjNmOHhJOUZIR0xzR1Z4NlVlUlBDZ0FpQXQ1R3JzZFFoT0tuWmNBM0VXZWN2Z0pTSHpDSWpXaWZGQlFrRDdIZHN5bWc9PSIsInB1YmxpY0tleSI6eyJjb250ZW50IjoiTFMwdExTMUNSVWRKVGlCRFJWSlVTVVpKUTBGVVJTMHRMUzB0Q2sxSlNVTnhSRU5EUVdrclowRjNTVUpCWjBsVlZGQlhWR1pQTHpGT1VtRlRSbVJsWTJGQlVTOXdRa1JIU25BNGQwTm5XVWxMYjFwSmVtb3dSVUYzVFhjS1RucEZWazFDVFVkQk1WVkZRMmhOVFdNeWJHNWpNMUoyWTIxVmRWcEhWakpOVWpSM1NFRlpSRlpSVVVSRmVGWjZZVmRrZW1SSE9YbGFVekZ3WW01U2JBcGpiVEZzV2tkc2FHUkhWWGRJYUdOT1RXcEplRTFVU1RGTlJHTjZUbnBGZVZkb1kwNU5ha2w0VFZSSk1VMUVZekJPZWtWNVYycEJRVTFHYTNkRmQxbElDa3R2V2tsNmFqQkRRVkZaU1V0dldrbDZhakJFUVZGalJGRm5RVVZLVVZFMFZ5ODFXRkE1YlRSWllsZFNRbEYwU0VkWGQyNDVkVlZvWVdVek9GVndZMG9LY0VWTk0wUlBjelI2VnpSTlNYSk5abGMwVjFGRU1HWjNjRGhRVlZWU1JGaDJVVE01TkhCdmNXZEhSVzFUYTNKMVRIRlBRMEZWTkhkblowWkxUVUUwUndwQk1WVmtSSGRGUWk5M1VVVkJkMGxJWjBSQlZFSm5UbFpJVTFWRlJFUkJTMEpuWjNKQ1owVkdRbEZqUkVGNlFXUkNaMDVXU0ZFMFJVWm5VVlZ2TTB0dUNtcEtVVm93V0dacFoySkVOV0l3VDFaT1RqQjRjVk52ZDBoM1dVUldVakJxUWtKbmQwWnZRVlV6T1ZCd2VqRlphMFZhWWpWeFRtcHdTMFpYYVhocE5Ga0tXa1E0ZDBwM1dVUldVakJTUVZGSUwwSkNNSGRITkVWYVdrZEdkV0ZYVm5OTWJVcHNaRzFXZFdGWVZucFJSMlIwV1Zkc2MweHRUblppVkVGelFtZHZjZ3BDWjBWRlFWbFBMMDFCUlVKQ1FqVnZaRWhTZDJONmIzWk1NbVJ3WkVkb01WbHBOV3BpTWpCMllrYzVibUZYTkhaaU1rWXhaRWRuZDJkWmMwZERhWE5IQ2tGUlVVSXhibXREUWtGSlJXWlJVamRCU0d0QlpIZEVaRkJVUW5GNGMyTlNUVzFOV2tob2VWcGFlbU5EYjJ0d1pYVk9ORGh5Wml0SWFXNUxRVXg1Ym5VS2FtZEJRVUZaVTNSMVFraDVRVUZCUlVGM1FrbE5SVmxEU1ZGRVRUVlpVMUV2UjB3MlMwazFVamxQWkdOdUwzQlRheXR4VmtRMlluTk1PRE1yUlhBNVVnb3lhRmRVWVhkSmFFRkxNV3BwTVd4YU5UWkVjMloxVEdaWU4ySkNRemx1WWxJelJXeDRZV3hDYUhZeGVsRllUVlUzZEd4M1RVRnZSME5EY1VkVFRUUTVDa0pCVFVSQk1tTkJUVWRSUTAxQ1N6aDBjMmRJWldkMWFDdFphR1ZzTTFCcGFraFJiSGxLTVZFMVN6WTBjREI0Y1VSa2J6ZFhOR1o0Wm05QlV6bDRjbEFLY3pKUVMxRmpaRzlFT1dKWWQyZEpkMWcyZWt4cWVXSmFhMDVJVURWNGRFSndOM1pMTWtaWlpWcDBNRTlYVEZKc1ZXeHNZMVZFVEROVUx6ZEtVV1ozWXdwSFUzRTJkbFpDVG5kS01EQjNPVWhTQ2kwdExTMHRSVTVFSUVORlVsUkpSa2xEUVZSRkxTMHRMUzBLIn19fX0=","integratedTime":1669361833,"logIndex":7810348,"logID":"c0d23d6ad406973f9559f3ba2d1ca01f84147d8ffc5b8445c224f98b9591801d"}}}
        "#;
        let rekor_pub_key = get_rekor_public_key();
        let result = SignedArtifactBundle::new_verified(&bundle_raw, &rekor_pub_key);
        assert!(result.is_ok());
        let bundle = result.unwrap();
        assert_eq!(bundle.rekor_bundle.payload.log_index, 7810348);
    }
}
