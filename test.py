from typing import Dict, Hashable

import hypothesis.strategies as st
from hypothesis.stateful import Bundle, RuleBasedStateMachine, rule

from graphlib2 import PyHashMap


class HashMapComparison(RuleBasedStateMachine):
    def __init__(self):
        super().__init__()
        self.python: Dict[Hashable, Hashable] = {}
        self.rust = 

    keys = Bundle("keys")
    values = Bundle("values")

    @rule(target=keys, k=st.binary())
    def add_key(self, k):
        return k

    @rule(target=values, v=st.binary())
    def add_value(self, v):
        return v

    @rule(k=keys, v=values)
    def save(self, k, v):
        self.model[k].add(v)
        self.database.save(k, v)

    @rule(k=keys, v=values)
    def delete(self, k, v):
        self.model[k].discard(v)
        self.database.delete(k, v)

    @rule(k=keys)
    def values_agree(self, k):
        assert set(self.database.fetch(k)) == self.model[k]

    def teardown(self):
        shutil.rmtree(self.tempd)


TestDBComparison = DatabaseComparison.TestCase
