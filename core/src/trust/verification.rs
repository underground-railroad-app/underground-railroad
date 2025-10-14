//! Verification methods for establishing trust

use crate::{CoarseTimestamp, Fingerprint, PersonId, SecureBytes};
use serde::{Deserialize, Serialize};

/// How was a contact verified?
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Met in person and verified fingerprint
    InPerson,

    /// Video call with fingerprint verification
    VideoCall,

    /// Phone call with fingerprint verification
    PhoneCall,

    /// Introduced by trusted contact
    Introduction,

    /// Verified via encrypted message exchange
    MessageExchange,

    /// Not yet verified
    Unverified,
}

/// Proof of verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationProof {
    /// Who verified whom?
    pub verifier: PersonId,
    pub verified: PersonId,

    /// Method used
    pub method: VerificationMethod,

    /// Fingerprint that was verified
    pub fingerprint: Fingerprint,

    /// When was verification performed?
    pub verified_at: CoarseTimestamp,

    /// Location of verification (if in-person, coarse)
    pub location: Option<String>,

    /// Additional proof data (encrypted)
    pub proof_data: Option<SecureBytes>,

    /// Signature from verifier
    pub signature: Option<SecureBytes>,
}

/// A verification request (asking someone to verify you)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    /// Who is requesting verification?
    pub requester: PersonId,

    /// Who should verify?
    pub verifier: PersonId,

    /// Proposed method
    pub method: VerificationMethod,

    /// Requester's fingerprint
    pub fingerprint: Fingerprint,

    /// When was this requested?
    pub requested_at: CoarseTimestamp,

    /// Status of request
    pub status: VerificationStatus,
}

/// Status of a verification request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Pending response
    Pending,

    /// Accepted, verification in progress
    Accepted,

    /// Completed successfully
    Completed,

    /// Rejected
    Rejected,

    /// Expired
    Expired,
}

impl VerificationProof {
    /// Create a new verification proof
    pub fn new(
        verifier: PersonId,
        verified: PersonId,
        method: VerificationMethod,
        fingerprint: Fingerprint,
    ) -> Self {
        Self {
            verifier,
            verified,
            method,
            fingerprint,
            verified_at: CoarseTimestamp::now(),
            location: None,
            proof_data: None,
            signature: None,
        }
    }

    /// Add location information
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }

    /// Add proof data
    pub fn with_proof_data(mut self, data: SecureBytes) -> Self {
        self.proof_data = Some(data);
        self
    }

    /// Add signature
    pub fn with_signature(mut self, signature: SecureBytes) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Check if this proof is for in-person verification
    pub fn is_in_person(&self) -> bool {
        self.method == VerificationMethod::InPerson
    }

    /// Check if this proof has a signature
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

impl VerificationRequest {
    /// Create a new verification request
    pub fn new(
        requester: PersonId,
        verifier: PersonId,
        method: VerificationMethod,
        fingerprint: Fingerprint,
    ) -> Self {
        Self {
            requester,
            verifier,
            method,
            fingerprint,
            requested_at: CoarseTimestamp::now(),
            status: VerificationStatus::Pending,
        }
    }

    /// Accept this verification request
    pub fn accept(&mut self) {
        if self.status == VerificationStatus::Pending {
            self.status = VerificationStatus::Accepted;
        }
    }

    /// Complete this verification
    pub fn complete(&mut self) {
        if self.status == VerificationStatus::Accepted {
            self.status = VerificationStatus::Completed;
        }
    }

    /// Reject this verification request
    pub fn reject(&mut self) {
        if self.status == VerificationStatus::Pending {
            self.status = VerificationStatus::Rejected;
        }
    }

    /// Check if request is still pending
    pub fn is_pending(&self) -> bool {
        self.status == VerificationStatus::Pending
    }

    /// Check if request is completed
    pub fn is_completed(&self) -> bool {
        self.status == VerificationStatus::Completed
    }
}

/// Verification checklist for in-person verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationChecklist {
    /// Met in person?
    pub met_in_person: bool,

    /// Verified fingerprint matches?
    pub fingerprint_verified: bool,

    /// Compared verification words?
    pub words_compared: bool,

    /// Verified identity (optional, depends on threat model)
    pub identity_checked: bool,

    /// Discussed security practices?
    pub security_discussed: bool,

    /// Established secure communication?
    pub secure_channel_established: bool,
}

impl VerificationChecklist {
    /// Create a new empty checklist
    pub fn new() -> Self {
        Self {
            met_in_person: false,
            fingerprint_verified: false,
            words_compared: false,
            identity_checked: false,
            security_discussed: false,
            secure_channel_established: false,
        }
    }

    /// Check if minimum requirements are met
    pub fn is_valid(&self) -> bool {
        // Minimum: met in person, fingerprint verified, words compared
        self.met_in_person && self.fingerprint_verified && self.words_compared
    }

    /// Check if all items are completed
    pub fn is_complete(&self) -> bool {
        self.met_in_person
            && self.fingerprint_verified
            && self.words_compared
            && self.identity_checked
            && self.security_discussed
            && self.secure_channel_established
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> u8 {
        let mut completed = 0;
        let total = 6;

        if self.met_in_person { completed += 1; }
        if self.fingerprint_verified { completed += 1; }
        if self.words_compared { completed += 1; }
        if self.identity_checked { completed += 1; }
        if self.security_discussed { completed += 1; }
        if self.secure_channel_established { completed += 1; }

        ((completed as f32 / total as f32) * 100.0) as u8
    }
}

impl Default for VerificationChecklist {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_proof() {
        let verifier = PersonId::new();
        let verified = PersonId::new();
        let fingerprint = Fingerprint::new([0u8; 32]);

        let proof = VerificationProof::new(
            verifier,
            verified,
            VerificationMethod::InPerson,
            fingerprint,
        )
        .with_location("Coffee Shop");

        assert!(proof.is_in_person());
        assert_eq!(proof.location, Some("Coffee Shop".to_string()));
    }

    #[test]
    fn test_verification_request() {
        let requester = PersonId::new();
        let verifier = PersonId::new();
        let fingerprint = Fingerprint::new([0u8; 32]);

        let mut request = VerificationRequest::new(
            requester,
            verifier,
            VerificationMethod::VideoCall,
            fingerprint,
        );

        assert!(request.is_pending());

        request.accept();
        assert_eq!(request.status, VerificationStatus::Accepted);

        request.complete();
        assert!(request.is_completed());
    }

    #[test]
    fn test_verification_checklist() {
        let mut checklist = VerificationChecklist::new();

        assert!(!checklist.is_valid());
        assert_eq!(checklist.completion_percentage(), 0);

        checklist.met_in_person = true;
        checklist.fingerprint_verified = true;
        checklist.words_compared = true;

        assert!(checklist.is_valid());
        assert_eq!(checklist.completion_percentage(), 50);

        checklist.identity_checked = true;
        checklist.security_discussed = true;
        checklist.secure_channel_established = true;

        assert!(checklist.is_complete());
        assert_eq!(checklist.completion_percentage(), 100);
    }

    #[test]
    fn test_verification_request_rejection() {
        let requester = PersonId::new();
        let verifier = PersonId::new();
        let fingerprint = Fingerprint::new([0u8; 32]);

        let mut request = VerificationRequest::new(
            requester,
            verifier,
            VerificationMethod::PhoneCall,
            fingerprint,
        );

        request.reject();
        assert_eq!(request.status, VerificationStatus::Rejected);
        assert!(!request.is_completed());
    }
}
