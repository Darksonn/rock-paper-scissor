package rps

import rps.models.MarkovChain
import rps.models.Ensemble

object Main {
  def MarkovPredictor() = Ensemble(Vector(new MarkovChain(2, 100), new MarkovChain(1, 100), new MarkovChain(1, 20), new MarkovChain(3, 1000)))
  val RockBot = () => new TrivialStrategy(Rock)
  val MarkovBot = () => new MaximizerStrategy("MarkovBot", MarkovPredictor())
  val AntiMarkovBot = () => new MaximizerStrategy("AntiMarkovBot", Ensemble(Vector(MarkovPredictor(), new StrategyPredictor(MarkovBot()))))
  val strats = Vector[() => Strategy](RockBot, MarkovBot, AntiMarkovBot)
  def main(args: Array[String]) = {
    val host = args(0)
    for (strat <- strats) {
      Client.start(strat, host, 4321)
    }
  }
}