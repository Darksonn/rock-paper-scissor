package rps.models

import rps._

class MarkovChain(length: Int, persistence: Int, memory: List[(Move, Move)]) extends Predictor {
  def this(length: Int, persistence: Int) = this(length, persistence, List())
  lazy val table = {
    val init = memory.take(length)
    val its = memory.tails.map(_.take(length+1)).filter(it => it.length > 1 && init.startsWith(it.tail))
    val count = Array.fill(3)(0)
    for (seq <- its) {
      count(seq.head._2.ordinal) += 1
    }
    count
  }
  lazy val total = table.sum
  def _prob(move: Move): Double = {
    (table(move.ordinal) + 1.0) / (total + 3.0)
  }
  def _update(self: Move, other: Move): Predictor = {
    new MarkovChain(length, persistence, ((self, other) :: memory).take(persistence))
  }
}