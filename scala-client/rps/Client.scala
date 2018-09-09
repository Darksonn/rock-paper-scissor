package rps

import java.io.Writer
import java.io.Reader
import java.net.InetAddress
import java.net.Socket
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.nio.charset.Charset

case class Server(out: Writer, in: Reader)
class Client(strategy: () => Strategy, server: Server) {
  // TODO: refactor
  def run(): Unit = {
    val move2Char = Map[Move, Char](Rock -> 'r', Paper -> 'p', Scissor -> 's')
    val char2Move = move2Char.map{ case (a, b) => (b, a) } ++ move2Char.map{ case (a, b) => (b.toUpper, a) }
    val name = strategy().name
    server.out.write(name.length.toChar)
    server.out.write(name + "\n")
    server.out.flush()
    val rand = new java.util.Random()
    while (true) {
      var strat = strategy()
      var it = ' '
      def _next() = {
        val ch = server.in.read()
        if (ch != -1) it = ch.toChar
        else it = 'x'
        it
      }
      def next() = {
        while (_next() == ' ') {
          server.out.write(' ')
          server.out.flush()
        }
        it
      }
      if (next() != 'n') {
        println(strat.name + " ended: " + it)
        return
      }
      var move = strat.pick(rand)
      server.out.write(move2Char(move))
      server.out.flush()
      while ("RPS" contains next()) {
        strat = strat.result(move, char2Move(it))
        move = strat.pick(rand)
        server.out.write(move2Char(move))
        server.out.flush()
      }
      if ("rps" contains it) strat = strat.result(move, char2Move(it))
    }
  }
}
object Client {
  def connect(strategy: () => Strategy, address: String, port: Int) = {
    val iaddr = InetAddress.getByName(address)
    val sock = new Socket(iaddr, port)
    val server = new Server(new OutputStreamWriter(sock.getOutputStream, Charset.forName("UTF-8")), new InputStreamReader(sock.getInputStream, Charset.forName("UTF-8")))
    val client = new Client(strategy, server)
    client
  }
  def start(strategy: () => Strategy, address: String, port: Int) = {
    val thread = new Thread() {
      override def run() = {
        connect(strategy, address, port).run()
      }
    }
    thread.start()
    thread
  }
}