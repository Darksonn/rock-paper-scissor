package rps

sealed trait Move {
  lazy val beats = Move.beats(this)
  lazy val loses = Move.loses(this)
  lazy val ordinal = Move.moves.indexOf(this)
}
case object Rock extends Move
case object Paper extends Move
case object Scissor extends Move
object Move {
  val moves = Vector[Move](Rock, Paper, Scissor)
  val beats = Map[Move, Move](Rock -> Scissor, Paper -> Rock, Scissor -> Paper)
  val loses = Map[Move, Move](Rock -> Paper, Paper -> Scissor, Scissor -> Rock)
}

// decides what move to make given a sequence of moves that have been made
trait Strategy {
  def name: String
  def _move: Map[Move, Double]
  def move = _move
  def _result(self: Move, other: Move): Strategy
  def result(self: Move, other: Move) = _result(self, other)
  def pick(rand: java.util.Random): Move = {
    var it = rand.nextDouble()
    var theMove = _move
    while(!theMove.isEmpty) {
      val (m, p) = theMove.head
      if (p >= it) return m
      else it -= p
    }
    return Move.moves(rand.nextInt(3))
  }
}
// predicts the moves that the opponent will make given a sequence of moves that have been made
trait Predictor {
  def _prob(move: Move): Double
  def prob(move: Move) = _prob(move)
  def _update(self: Move, other: Move): Predictor
  def update(self: Move, other: Move) = _update(self, other)
}
// picks the move that maximizes the expected score
class MaximizerStrategy(val name: String, model: Predictor) extends Strategy {
  def _move = Map(Move.moves.maxBy(m => model.prob(m.beats) - model.prob(m.loses)) -> 1.0)
  def _result(self: Move, other: Move) = new MaximizerStrategy(name, model.update(self, other))
}
// predicts that the opponent will make the move that the given strategy would make
class StrategyPredictor(strat: Strategy) extends Predictor {
  def _prob(move: Move) = strat.move.get(move).getOrElse(0.0)
  def _update(self: Move, other: Move) = new StrategyPredictor(strat.result(other, self))
}
class TrivialStrategy(pick: Move) extends Strategy {
  def name = pick+"Bot"
  def _move = Map(pick -> 1.0)
  def _result(self: Move, other: Move) = this
}