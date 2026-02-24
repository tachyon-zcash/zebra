# Cryptographic Assumptions

Ragu's security rests on several standard cryptographic assumptions. This page
documents each assumption and its role in the protocol.

## Elliptic-Curve Discrete Logarithm {#ecdlp}

The **elliptic-curve discrete logarithm problem (ECDLP)** is: given points
$G, P \in \G$ where $P = a \cdot G$ for some unknown scalar $a \in \F$, find
$a$. No efficient algorithm exists for this problem on well-chosen elliptic
curves, and breaking it would require undermining the foundation of all
elliptic-curve cryptography.

In practice, Ragu relies on a stronger variant to ensure the security of its
commitment scheme: the **discrete log relation assumption**. Given a vector of
independently chosen generators $\v{G} \in \G^n$, it is infeasible to find a
non-trivial vector of scalars $\v{a} \in \F^n$ such that
$\dot{\v{a}}{\v{G}} = \identity$. This is stronger because finding such a
relation for any pair of generators would break ECDLP. This assumption underlies
the binding property of the Pedersen vector commitments that Ragu uses for
polynomial commitment, and by extension the soundness of the entire proof
system.

## Random Oracle Model {#random-oracle-model}

Ragu models [Poseidon](https://eprint.iacr.org/2019/458) as a **random oracle**
in the Fiat-Shamir transformation that makes the proof protocol non-interactive.
This is a stronger assumption than collision resistance alone: we require that
Poseidon's outputs are indistinguishable from uniformly random values, so that
challenges derived from the [transcript](transcript.md) behave as if chosen by
an honest verifier.

## Knowledge Soundness {#knowledge-soundness}

Ragu's polynomial commitment scheme (PCS) relies on a **knowledge assumption**:
an efficient prover who produces a valid commitment and evaluation proof must
"know" the underlying polynomial. Formally, there exists a knowledge extractor
that can recover the committed polynomial from any successful prover's internal
state. This assumption is standard for inner-product-argument-based PCS
constructions and is what makes the NARK a _non-interactive argument of
knowledge_ rather than merely an argument of correctness.
