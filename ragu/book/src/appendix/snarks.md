# SNARKs

A **succinct non-interactive argument of knowledge (SNARK)** is a
cryptographic proof system that allows a prover to convince a verifier
that a statement is true, without revealing any information beyond the
validity of the statement itself. The key properties are:

- **Succinct**: the proof is small (much smaller than the computation
  it attests to) and fast to verify.
- **Non-interactive**: the prover sends a single message to the
  verifier; no back-and-forth is needed.
- **Argument of knowledge**: the prover not only demonstrates that a
  valid witness _exists_, but that they actually _know_ one.

SNARKs are the building blocks of
[proof-carrying data](../concepts/pcd.md): each step in a PCD computation
produces a SNARK that attests to the correctness of that step, and
recursive composition allows these proofs to reference each other.
