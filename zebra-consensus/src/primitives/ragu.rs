//! Async Ragu batch verifier service for Tachyon proofs.
//!
//! This is a stub/placeholder batch verifier that mirrors the structure of the
//! Halo2 batch verifier. Real Ragu PCD verification will be implemented once
//! the proof system is ready.

use std::{
    fmt,
    future::Future,
    mem,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{future::BoxFuture, FutureExt};
use once_cell::sync::Lazy;
use thiserror::Error;
use tokio::sync::watch;
use tower::{util::ServiceFn, Service};
use tower_batch_control::{Batch, BatchControl, RequestWeight};
use tower_fallback::Fallback;
use zcash_tachyon::{Bundle, Stamp};

use crate::BoxError;

use super::spawn_fifo;

/// Adjusted batch size for ragu batches.
///
/// Like Halo2, Tachyon has aggregate proofs, so we weight by action count.
const RAGU_MAX_BATCH_SIZE: usize = super::MAX_BATCH_SIZE;

/// The type of verification results.
type VerifyResult = bool;

/// The type of the batch sender channel.
type Sender = watch::Sender<Option<VerifyResult>>;

/// A Ragu verification item, used as the request type of the service.
#[derive(Clone, Debug)]
pub struct Item {
    bundle: Bundle<Stamp>,
}

impl RequestWeight for Item {
    fn request_weight(&self) -> usize {
        self.bundle.actions.len()
    }
}

impl Item {
    /// Creates a new [`Item`] from a stamped Tachyon bundle.
    pub fn new(bundle: Bundle<Stamp>) -> Self {
        Self { bundle }
    }

    /// Perform non-batched verification of this [`Item`].
    ///
    /// This is a stub that always returns `true`. Real Ragu PCD verification
    /// will be implemented once the proof system is ready.
    pub fn verify_single(self) -> bool {
        // TODO: Call real Ragu verification once zcash_tachyon::Proof::verify
        // is implemented. The verification steps will be:
        // 1. Recompute action_acc from bundle actions
        // 2. Recompute tachygram_acc from stamp tachygrams
        // 3. Construct PCD header (action_acc, tachygram_acc, anchor)
        // 4. Call ragu verify(Pcd { proof, data: header })
        let _proof = &self.bundle.stamp.proof;
        let _actions = &self.bundle.actions;
        let _tachygrams = &self.bundle.stamp.tachygrams;
        let _anchor = self.bundle.stamp.anchor;
        true
    }
}

/// An error that may occur when verifying Ragu proofs of Tachyon transactions.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[allow(missing_docs)]
pub enum RaguError {
    #[error("Ragu PCD proof verification failed")]
    ProofVerificationFailure,
    #[error("unknown Ragu error")]
    Other,
}

/// Global batch verification context for Ragu proofs of Tachyon transactions.
///
/// This service transparently batches contemporaneous proof verifications,
/// handling batch failures by falling back to individual verification.
///
/// Note that making a `Service` call requires mutable access to the service, so
/// you should call `.clone()` on the global handle to create a local, mutable
/// handle.
pub static VERIFIER: Lazy<
    Fallback<
        Batch<Verifier, Item>,
        ServiceFn<fn(Item) -> BoxFuture<'static, Result<(), BoxError>>>,
    >,
> = Lazy::new(|| {
    Fallback::new(
        Batch::new(
            Verifier::new(),
            RAGU_MAX_BATCH_SIZE,
            None,
            super::MAX_BATCH_LATENCY,
        ),
        tower::service_fn(
            (|item: Item| Verifier::verify_single_spawning(item).boxed()) as fn(_) -> _,
        ),
    )
});

/// Ragu proof verifier implementation.
///
/// This is the core implementation for the batch verification logic of the
/// Ragu verifier. It handles batching incoming requests, driving batches to
/// completion, and reporting results.
///
/// Currently a stub — real batch verification will be added once Ragu PCD
/// is implemented.
pub struct Verifier {
    /// Pending items for batch verification.
    ///
    /// TODO: Replace with a proper Ragu BatchValidator type once the
    /// proof system provides one.
    batch: Vec<Item>,

    /// A channel for broadcasting the result of a batch to the futures for each batch item.
    ///
    /// Each batch gets a newly created channel, so there is only ever one result sent per channel.
    tx: Sender,
}

impl Verifier {
    fn new() -> Self {
        let (tx, _) = watch::channel(None);
        Self {
            batch: Vec::new(),
            tx,
        }
    }

    /// Returns the batch and channel sender from `self`,
    /// replacing them with a new empty batch.
    fn take(&mut self) -> (Vec<Item>, Sender) {
        let batch = mem::take(&mut self.batch);

        let (tx, _) = watch::channel(None);
        let tx = mem::replace(&mut self.tx, tx);

        (batch, tx)
    }

    /// Synchronously process the batch, and send the result using the channel sender.
    fn verify(batch: Vec<Item>, tx: Sender) {
        // Stub: verify each item individually and AND the results.
        let result = batch.into_iter().all(|item| item.verify_single());
        let _ = tx.send(Some(result));
    }

    /// Flush the batch using a thread pool, and return the result via the channel.
    /// This returns immediately, usually before the batch is completed.
    fn flush_blocking(&mut self) {
        let (batch, tx) = self.take();

        // Correctness: Do CPU-intensive work on a dedicated thread, to avoid blocking other futures.
        tokio::task::block_in_place(|| rayon::spawn_fifo(|| Self::verify(batch, tx)));
    }

    /// Flush the batch using a thread pool, and return the result via the channel.
    /// This function returns a future that becomes ready when the batch is completed.
    async fn flush_spawning(batch: Vec<Item>, tx: Sender) {
        // Correctness: Do CPU-intensive work on a dedicated thread, to avoid blocking other futures.
        let _ = tx.send(
            spawn_fifo(move || batch.into_iter().all(|item| item.verify_single()))
                .await
                .ok(),
        );
    }

    /// Verify a single item using a thread pool, and return the result.
    async fn verify_single_spawning(item: Item) -> Result<(), BoxError> {
        if spawn_fifo(move || item.verify_single()).await? {
            Ok(())
        } else {
            Err("could not validate ragu proof".into())
        }
    }
}

impl fmt::Debug for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Verifier")
            .field("batch_len", &self.batch.len())
            .field("tx", &self.tx)
            .finish()
    }
}

impl Service<BatchControl<Item>> for Verifier {
    type Response = ();
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<(), BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: BatchControl<Item>) -> Self::Future {
        match req {
            BatchControl::Item(item) => {
                tracing::trace!("got ragu item");
                self.batch.push(item);
                let mut rx = self.tx.subscribe();
                Box::pin(async move {
                    match rx.changed().await {
                        Ok(()) => {
                            let is_valid = *rx
                                .borrow()
                                .as_ref()
                                .ok_or("threadpool unexpectedly dropped response channel sender. Is Zebra shutting down?")?;

                            if is_valid {
                                tracing::trace!(?is_valid, "verified ragu proof");
                                metrics::counter!("proofs.ragu.verified").increment(1);
                                Ok(())
                            } else {
                                tracing::trace!(?is_valid, "invalid ragu proof");
                                metrics::counter!("proofs.ragu.invalid").increment(1);
                                Err("could not validate ragu proofs".into())
                            }
                        }
                        Err(_recv_error) => panic!("verifier was dropped without flushing"),
                    }
                })
            }

            BatchControl::Flush => {
                tracing::trace!("got ragu flush command");

                let (batch, tx) = self.take();

                Box::pin(Self::flush_spawning(batch, tx).map(Ok))
            }
        }
    }
}

impl Drop for Verifier {
    fn drop(&mut self) {
        // We need to flush the current batch in case there are still any pending futures.
        self.flush_blocking()
    }
}
