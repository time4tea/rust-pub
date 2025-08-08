Experimental replacement for 'dart pub'

Not working yet - the goal is to make a much faster version of dart pub so that:

- the test runner when it invokes it a lot will get a fasster result
- it downloads mono-repo dependencies faster and in parallel
- it writes the various dart files much faster.

