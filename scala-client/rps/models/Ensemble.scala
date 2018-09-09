package rps.models

import rps._

class Ensemble(model: Vector[(Predictor, Double)], decay: Double) extends Predictor {
  def _prob(move: Move): Double = {
    model.map { case (m, p) => m.prob(move) * p }.sum
  }

  def _update(self: Move, other: Move): Predictor = {
    val preds = model.map { case (model, prob) => (model.update(self, other), prob, model.prob(other)) }
    val likelihood = preds.map(_._3).sum
    val updated = preds.map { case (model, prob, pred) => (model, pred * prob / likelihood) }
    val escort = updated.map { case (model, prob) => (model, Math.pow(prob, decay)) }
    val sum = escort.map(_._2).sum
    val norm = escort.map { case (model, prob) => (model, prob/sum) }
    new Ensemble(norm, decay)
  }
}
object Ensemble {
  def apply(predictors: Vector[Predictor], decay: Double = 0.9) = new Ensemble(predictors.map((_, 1.0/predictors.length)), decay)
}