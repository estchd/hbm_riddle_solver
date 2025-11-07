# HBM NTM Riddle Solver

An attempt at semi-brute-forcing the answers to the four HBM NTM Hidden Catalog riddles.

This solver can check single solutions, try all combinations out of a set of answers and brute force.
How each question is solved can be configured on a per-question basis.

Sadly, since everything is hashed into a single hash, validating questions separately is not possible so it is impossible to tell which of your answers is wrong.

## Validity

I created this validation logic of this solver by copying the HBM Riddle validation logic into a separate Java project and reimplementing all of the pertinent logic in rust while validating that the rust solution matches the output of the java solution at each validation step.
Most of the step by step validations are no longer present in code (but can still be found in the git history) but i have done spot checks to ensure that the output hashes match.

## Complexity

Even though this started trying to Brute force the hash, Brute Forcing, even of a single answer is infeasable.

A single answer can have 15 symbols. Even for the absolute minimum character list that might not even be correct, that is 27^15 or ~3.000.000.000.000.000.000.000 possible answers.
On my relatively high end pc, this solver can check ~1.000.000 answers per second (not per thread per second, per second).
While this could probably go a magnitude higher by moving the computation to the gpu, it still would take waaay too long.

## Using this yourself

If you want to use this, you need cargo installed and just run `cargo run` in the main directory.
There are a few cargo features that can configure how the solver works:

- `hotpath`: When this feature is enabled, the solver only runs 1 second and connects some performance characteristics. You most likely won't need this, ever 
