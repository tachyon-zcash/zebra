# Notation

## Groups and Fields

Group elements are written in uppercase letters, e.g. $G, H, P\in\G$,
scalars and field elements are in lowercase, e.g. $a, b, c\in\F$.

## Vectors

We write a vector $\v{a} \in \F^n$ in bold type, and generally use capital
letters like $\v{G} \in \mathbb{G}^n$ to represent vectors of group elements.
All vectors are zero-indexed.
Vector concatenation is denoted as $\v{a}\|\v{b}\in\F^{2n}$.

Dot (inner) products as $\dot{\v{a}}{\v{b}}=\sum_i \v{a}_i\cdot\v{b}_i \in\F$
and Hadamard (pair-wise) products as $\v{a} \circ \v{b}\in\F^n$.

We use $\v{z^{n}}$ to denote power vector $(z^0, z^1, \cdots, z^{n - 1})$.
One natural exception in notation: 
$\v{0^n}=(\underbrace{0,\ldots,0}_{n\text{ zeros}})$ is a zero vector, 
not $(1,0,\ldots)$ even though $0^0=1$. 

### Reversed Vector

Given a vector $\v{a} \in \F^n$ we denote its **reverse (mirror)** as
$\v{\hat{a}}$
such that $\v{\hat{a}}_i = \v{a}_{n - 1 - i} \forall i$.

### Revdot Product

We use a special notation $\revdot{\v{a}}{\v{b}}$ for 
$\dot{\v{a}}{\rv{b}} = \dot{\rv{a}}{\v{b}}$,
which we referred to as **revdot products**, a special case of dot products.


## Polynomials

Given a univariate polynomial $p \in \F[X]$ of maximal degree $n - 1$ there
exists a unique coefficient vector $\v{p} = (p_0, p_1, \ldots, p_{n-1})
\in \F^n$ where $p(X) = \sum_{i=0}^{n-1} p_i X^i$, ordered from lowest to
highest degree.

* **Evaluation:** Given $z \in \F$ the evaluation $p(z)$ is given by the inner
  (dot) product $\langle \v{p}, \v{z^n} \rangle$.
* **Dilation:** We write the dilation $p(zX)$ using the Hadamard (pairwise)
  product $\v{z^n} \circ \v{p}$.
* **Reversal:** The reversed coefficient vector $\rv{p}$ represents the
  polynomial $\hat{p}(X) = X^{n-1} p(X^{-1})$.
