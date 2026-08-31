# Problem Modeling Template

## Use this template to convert a real-world problem into a DP formulation

---

## PROBLEM
--------
**What is the real-world problem?**

[Describe the problem in natural language]


## OBJECTIVE
---------
**What are we optimizing?**

[Define the goal clearly]


## TIME
----
**Is there a sequence of decisions?**

[Yes/No, describe if applicable]


## DECISIONS
---------
**What can the agent control?**

[List controllable decisions/actions]


## STATE
-----
**What information changes?**

[List all changing variables]


## STATE SUFFICIENCY
-----------------
**Does the state contain everything necessary to determine future possibilities?**

[Test: If I delete the entire history before time t, can I still determine all possible future transitions and their consequences from S_t?]


## ACTIONS
-------
**What actions are legal from each state?**

[Define action space and constraints]


## CONSTRAINTS
-----------
**What limits the actions?**

[List all constraints]


## TRANSITION
----------
**How does an action transform the state?**

[Define transition function]


## REWARD
------
**What is gained/lost?**

[Define objective function/reward]


## TERMINAL CONDITION
------------------
**When does the problem end?**

[Define terminal condition]


## OVERLAPPING SUBPROBLEMS
-----------------------
**Can different histories reach the same state?**

[Yes/No, with explanation]


## OPTIMAL SUBSTRUCTURE
--------------------
**Can the optimal solution be composed from optimal solutions of smaller future problems?**

[Yes/No, with explanation]


## STATE SIZE
----------
**How many possible states exist?**

[Calculate or estimate state count]


## ACTION SIZE
-----------
**How many actions exist per state?**

[Calculate or estimate action count]


## COMPLEXITY
----------
**What is the resulting time/space complexity?**

[Derive Big-O complexity]


## APPROXIMATION
-------------
**If exact DP is too large, what can be compressed?**

[Identify possible state/action compressions]


## VALIDATION
----------
**How do we know the resulting strategy is good?**

[Describe validation methodology]


---

## NOTES
-------
**Additional considerations:**

[Any other relevant information]


## CONCLUSION
-----------
**Is this a DP problem?**

[Yes/No/Approximate DP with limitations]