#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;

use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use std::iter::once;

use seahash::SeaHasher;

use crate::auto::{Automaton, StateId};
use crate::{Constructor, Label, Polarity};

impl<C: Constructor> Automaton<C> {
    /// Solves a constraint t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    #[must_use]
    pub fn biunify(&mut self, qp: StateId, qn: StateId) -> Result<(), ()> {
        self.biunify_all(once((qp, qn)))
    }

    #[must_use]
    pub fn biunify_all<I>(&mut self, constraints: I) -> Result<(), ()>
    where
        I: IntoIterator<Item = (StateId, StateId)>,
    {
        let mut seen = HashSet::with_capacity_and_hasher(20, Default::default());
        constraints
            .into_iter()
            .try_for_each(|(qp, qn)| self.biunify_impl(&mut seen, qp, qn))
    }

    fn biunify_impl(
        &mut self,
        seen: &mut HashSet<(StateId, StateId), BuildHasherDefault<SeaHasher>>,
        qp: StateId,
        qn: StateId,
    ) -> Result<(), ()> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qp].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qn].pol, Polarity::Neg);

        if seen.insert((qp, qn)) {
            if !product(&self[qp].cons, &self[qn].cons).all(|(l, r)| l <= r) {
                return Err(());
            }
            for to in self[qn].flow.iter() {
                self.merge(Polarity::Pos, to, qp);
            }
            for from in self[qp].flow.iter() {
                self.merge(Polarity::Neg, from, qn);
            }
            let jps = self[qp].cons.clone();
            let jns = self[qn].cons.clone();
            for (label, l, r) in jps.intersection(jns) {
                let (ps, ns) = label.polarity().flip(l, r);
                for (jp, jn) in product(ps, ns) {
                    self.biunify_impl(seen, jp, jn)?;
                }
            }
        }
        Ok(())
    }
}

fn product<I, J>(lhs: I, rhs: J) -> impl Iterator<Item = (I::Item, J::Item)>
where
    I: IntoIterator,
    I::Item: Clone,
    J: IntoIterator,
    J: Clone,
{
    lhs.into_iter()
        .flat_map(move |l| rhs.clone().into_iter().map(move |r| (l.clone(), r)))
}
