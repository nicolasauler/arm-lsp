# CODEGEN

Esse diretório contém o código fonte do gerador de código ARM Assembly a partir de operações matemáticas.

## Instalação

Instale a biblioteca PLY:

```bash
pip install -r requirements.txt
```

Depois é só rodar:

```bash
python main.py
```

## Como usar

edite o arquivo math.txt com a expressão matemática desejada

operaçoes matemáticas suportadas:

- soma (+)
- subtração (-)
- multiplicação (*)
- divisão (/)
- módulo (%)

também suportamos if/else e operadores ternários como:
```
if (a < b) then f=50 else f=30
```

```
z > g ? f=5 : f=15
```

para definir uma variável, use nome=valor


exemplo:
```
a=8
b=3
c=4
d=3
g=a+(b+c)
if (a < b) then f=50 else f=30
z = 99
z > g ? f=5 : f=15
```

## Descrição do Código

### Importações e Variáveis Globais:

ply.lex e ply.yacc: São módulos do pacote PLY para construção de analisadores léxicos e sintáticos.

Variáveis como ASM, name_to_register, stack etc. são usadas globalmente para armazenar informações relevantes durante a análise. ASM é uma lista de strings que armazena o código Assembly gerado, name_to_register é um dicionário que mapeia nomes de variáveis para registradores, stack é pilha auxiliar para a resolução de expressões tipo if-then-else

### Tokenizer (Analisador Léxico):

Esta parte do código define como o texto de entrada é dividido em tokens. A lista tokens define todos os tipos de tokens que o programa pode reconhecer.
A lista literals define os operadores aritméticos.
As funções começando com t_ definem como reconhecer e processar esses tokens.

### Parser (Analisador Sintático):

O analisador sintático define como os tokens são agrupados em expressões e instruções. Por exemplo, uma expressão pode consistir em outra expressão seguida por um '+' e outra expressão.
O método yacc.yacc() constrói o analisador sintático.
As funções que começam com p_ definem as produções da gramática e o que fazer quando essa produção é reconhecida. Por exemplo, p_statement_assign traduz uma instrução de atribuição (como a = 5) para o código Assembly ARM correspondente.
A função p_error é chamada se um erro sintático é encontrado.
Tradução para Assembly:

Funções como add_instruction, handle_condition, get_free_reg etc. são funções auxiliares que ajudam a gerar o código Assembly ARM correspondente.
handle_condition é particularmente interessante porque traduz operadores de comparação como '<' e '>' para as instruções ARM condicionais.
As produções da gramática no analisador sintático (funções p_) invocam essas funções auxiliares para gerar o código Assembly.
Main e I/O:

### Main

A função main abre o arquivo de entrada math.txt, analisa-o linha por linha e imprime o código Assembly gerado.
O código Assembly gerado é salvo em output.s.
Foco no processo de tradução:
Quando uma expressão ou instrução é reconhecida, as funções de produção da gramática (p_) são chamadas. Estas, por sua vez, chamam funções auxiliares que geram o código Assembly ARM.

Por exemplo, considerando a expressão a + b:

A função p_expression_math é chamada quando a expressão é reconhecida.
Esta função verifica qual operador foi usado (+ neste caso) e chama add_instruction com a instrução ARM correspondente (ADD).

A parte mais complexa é provavelmente a gestão de registradores (get_free_reg, name_to_register, etc.), uma vez que o código garante que cada variável e resultado intermediário tem seu próprio registrador.