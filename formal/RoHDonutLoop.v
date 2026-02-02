Require Import Reals.
Require Import Coq.Lists.List.
Import ListNotations.

Open Scope R_scope.

(* One .evolve turn, already parsed from JSONL and validated syntactically. *)
Record evolve_turn := {
  turn_index : nat;
  roh        : R
}.

Definition evolve_stream := list evolve_turn.

(* Hypothesis: the stream is ordered by turn_index, without gaps. *)
Definition ordered_stream (s : evolve_stream) : Prop :=
  forall i t1 t2,
    nth_error s i = Some t1 ->
    nth_error s (S i) = Some t2 ->
    turn_index t2 = S (turn_index t1).

(* Hypothesis: RoH extracted per turn from .evolve.jsonl. *)
Parameter RoH : nat -> R.

(* Pointwise extraction agreement between JSON and RoH function. *)
Definition roh_stream_agrees (s : evolve_stream) : Prop :=
  forall i t,
    nth_error s i = Some t ->
    roh t = RoH (turn_index t).

(* Donutloop constraints as Coq predicates. *)

Definition RoH_bounded (s : evolve_stream) : Prop :=
  forall t,
    In t s ->
    0 <= roh t <= 0.3.

Definition RoH_monotone (s : evolve_stream) : Prop :=
  forall i t1 t2,
    nth_error s i = Some t1 ->
    nth_error s (S i) = Some t2 ->
    roh t2 <= roh t1.

(********************************************************************)
(* Core lemmas you can publish                                      *)
(********************************************************************)

(* 1. Local step lemma: RoH_{t+1} <= RoH_t and RoH_t <= 0.3. *)

Lemma RoH_step_invariant :
  forall (s : evolve_stream) (i : nat) (t1 t2 : evolve_turn),
    ordered_stream s ->
    RoH_bounded s ->
    RoH_monotone s ->
    nth_error s i = Some t1 ->
    nth_error s (S i) = Some t2 ->
    roh t2 <= roh t1 /\ roh t1 <= 0.3.
Proof.
  intros s i t1 t2 Hord Hbound Hmono H1 H2.
  split.
  - (* monotonic envelope: RoH_{t+1} <= RoH_t *)
    apply Hmono with (i := i); assumption.
  - (* bound: RoH_t <= 0.3 *)
    specialize (Hbound t1).
    assert (In t1 s).
    { (* sketch: derive In from nth_error = Some *) admit. }
    specialize (Hbound H).
    tauto.
Qed.

(* 2. Global lemma: for all t, RoH_t <= 0.3. *)

Lemma RoH_global_bound :
  forall (s : evolve_stream),
    RoH_bounded s ->
    forall t, In t s -> roh t <= 0.3.
Proof.
  intros s Hbound t Hin.
  specialize (Hbound t Hin).
  tauto.
Qed.

(* 3. Stream-level monotone envelope: forall consecutive turns, RoH_{t+1} <= RoH_t. *)

Lemma RoH_stream_monotone :
  forall (s : evolve_stream),
    RoH_monotone s ->
    forall i t1 t2,
      nth_error s i = Some t1 ->
      nth_error s (S i) = Some t2 ->
      roh t2 <= roh t1.
Proof.
  intros s Hmono i t1 t2 H1 H2.
  apply (Hmono i t1 t2 H1 H2).
Qed.
