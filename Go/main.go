package main

import (
	"bufio"
	"fmt"
	"net"
	"strings"
	"strconv"
	"sync"
)

const PORT = "127.0.0.1:54322"

type TupleSpace struct {
	data map[string][]string
	mu   sync.Mutex
	cond *sync.Cond
}

func NewTupleSpace() *TupleSpace {
	ts := &TupleSpace{
		data: make(map[string][]string),
	}
	ts.cond = sync.NewCond(&ts.mu)
	return ts
}

// ======================
// Serviços
// ======================

func applyService(id int, input string) (string, bool) {
	switch id {
	case 1:
		return strings.ToUpper(input), true
	case 2:
		runes := []rune(input)
		for i, j := 0, len(runes)-1; i < j; i, j = i+1, j-1 {
			runes[i], runes[j] = runes[j], runes[i]
		}
		return string(runes), true
	case 3:
		return strconv.Itoa(len(input)), true
	default:
		return "", false
	}
}

// ======================
// Operações bloqueantes
// ======================

func (ts *TupleSpace) waitAndPeek(key string) string {
	ts.mu.Lock()
	defer ts.mu.Unlock()

	for {
		if queue, ok := ts.data[key]; ok && len(queue) > 0 {
			return queue[0]
		}
		ts.cond.Wait()
	}
}

func (ts *TupleSpace) waitAndPop(key string) string {
	ts.mu.Lock()
	defer ts.mu.Unlock()

	for {
		if queue, ok := ts.data[key]; ok && len(queue) > 0 {
			value := queue[0]
			ts.data[key] = queue[1:]
			if len(ts.data[key]) == 0 {
				delete(ts.data, key)
			}
			return value
		}
		ts.cond.Wait()
	}
}

func (ts *TupleSpace) write(key, value string) {
	ts.mu.Lock()
	defer ts.mu.Unlock()

	ts.data[key] = append(ts.data[key], value)
	ts.cond.Broadcast()
}

// ======================
// Cliente
// ======================

func handleClient(conn net.Conn, ts *TupleSpace) {
	defer conn.Close()

	reader := bufio.NewScanner(conn)

	for reader.Scan() {
		line := reader.Text()
		parts := strings.Fields(line)

		if len(parts) == 0 {
			fmt.Fprintln(conn, "ERROR")
			continue
		}

		switch parts[0] {

		case "WR":
			if len(parts) < 3 {
				fmt.Fprintln(conn, "ERROR")
				continue
			}

			key := parts[1]
			value := strings.Join(parts[2:], " ")
			ts.write(key, value)
			fmt.Fprintln(conn, "OK")

		case "RD":
			if len(parts) != 2 {
				fmt.Fprintln(conn, "ERROR")
				continue
			}

			value := ts.waitAndPeek(parts[1])
			fmt.Fprintf(conn, "OK %s\n", value)

		case "IN":
			if len(parts) != 2 {
				fmt.Fprintln(conn, "ERROR")
				continue
			}

			value := ts.waitAndPop(parts[1])
			fmt.Fprintf(conn, "OK %s\n", value)

		case "EX":
			if len(parts) != 4 {
				fmt.Fprintln(conn, "ERROR")
				continue
			}

			kIn := parts[1]
			kOut := parts[2]

			svcID, err := strconv.Atoi(parts[3])
			if err != nil {
				fmt.Fprintln(conn, "ERROR")
				continue
			}

			value := ts.waitAndPop(kIn)

			if result, ok := applyService(svcID, value); ok {
				ts.write(kOut, result)
				fmt.Fprintln(conn, "OK")
			} else {
				fmt.Fprintln(conn, "NO-SERVICE")
			}

		default:
			fmt.Fprintln(conn, "ERROR")
		}
	}
}

// ======================
// Main
// ======================

func main() {
	listener, err := net.Listen("tcp", PORT)
	if err != nil {
		panic("Erro ao abrir porta")
	}
	defer listener.Close()

	fmt.Println("Servidor Go rodando na porta 54322...")

	ts := NewTupleSpace()

	for {
		conn, err := listener.Accept()
		if err != nil {
			continue
		}

		go handleClient(conn, ts)
	}
}
