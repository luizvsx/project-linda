# Projeto Final — Servidor de Espaço de Tuplas (Linda)

Disciplina: Programação Concorrente  

Este projeto implementa um servidor concorrente inspirado no modelo Linda,
oferecendo um espaço de tuplas acessível via TCP.

Foram desenvolvidas duas versões independentes:

- Rust
- Go

Ambas seguem exatamente a mesma semântica e protocolo.

---

# 📌 Modelo de Tupla

Cada tupla possui o formato:

(chave: string, valor: string)

- A chave é uma string arbitrária.
- O valor é uma string arbitrária.
- O espaço pode conter múltiplas tuplas com a mesma chave.
- A política é FIFO por chave.

---

# 🔧 Operações Implementadas

## WR chave valor
Insere a tupla no espaço.  
Não bloqueia.  
Retorna:

OK

---

## RD chave
Leitura não destrutiva.  
Bloqueia até existir tupla com essa chave.  
Retorna:

OK valor

---

## IN chave
Leitura destrutiva (remove).  
Bloqueia até existir tupla com essa chave.  
Retorna:

OK valor

---

## EX chave_entrada chave_saida svc_id

1. Bloqueia até existir tupla com chave_entrada.
2. Remove a tupla (como IN).
3. Aplica o serviço correspondente.
4. Insere (chave_saida, resultado).

Retornos possíveis:

OK  
NO-SERVICE  

---

# 🧠 Serviços Implementados

| ID | Serviço |
|----|---------|
| 1  | Converter para maiúsculas |
| 2  | Inverter string |
| 3  | Retornar tamanho da string |

---

# 🌐 Protocolo TCP

Comandos enviados via texto:

WR chave valor  
RD chave  
IN chave  
EX chave_in chave_out svc_id  

Respostas:

OK  
OK valor  
NO-SERVICE  
ERROR  

---

# 🚀 Execução — Rust

## Compilar

```bash
#rust
cargo build --release

#Go
go mod init linda_go


Executar
porta utilizada: 127.0.0.1:54321

#Rust
cargo run

#Go
go run main.go

Teste:

g++ -std=c++17 tester_linda.cpp -o tester_linda

./tester_linda 127.0.0.1 54321   # Rust
./tester_linda 127.0.0.1 54322   # Go



