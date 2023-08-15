import ply.lex as lex
import ply.yacc as yacc

# stores the generated assembly code (list of instructions)
ASM = []

name_to_register = {}  # dictionary that maps name -> register
stack = []  # helper stack for control flow statements;eg: if-then-else

# internal variables
registers_in_use_internally = []  # registers being used for storing calculation results
current_register = ""  # current register

no_of_regs = 13


def add_instruction(instruction):
    ASM.append("\t" + instruction + "\n")


tokens = (
    "NAME",
    "NUMBER",
    "IF",
    "THEN",
    "ELSE",
)

# arithmetic operators
literals = ["=", "+", "-", "*", "<", ">", "/", "%", "(", ")", "?", ":"]

# skip tabs as spaces
t_ignore = " \t"

# how a "name" looks like. eg: a, abc, deadbeef, myname
t_NAME = r"[a-zA-Z_][a-zA-Z0-9_]*"


def t_NUMBER(t):
    r"-?\d+"
    t.value = int(t.value)
    return t


def t_IF(t):
    r"if"
    return t


def t_ELSE(t):
    r"else"
    return t


def t_THEN(t):
    r"then"
    return t


def t_newline(t):
    r"\n+"


def t_error(t):
    raise TypeError("Unknown text '%s'" % (t.value[0]))


# tokenizer
lex.lex()


# lowest to highest (internal PLY variable)
precedence = (
    ("left", ">", "<"),
    ("left", "+", "-"),
    ("left", "*", "/", "%"),
)


# returns a register string (like "r1")
def get_free_reg():
    for i in range(no_of_regs):
        # r1, r2, etc
        r = "r" + str(i)
        if (
            (r not in list(name_to_register.values()))
            and (r not in registers_in_use_internally)
            and (r != current_register)
        ):
            return r
    raise Exception("No free registers available")


# handles instructions for control flow statements (< and >)
def handle_condition(op, stack_length, name, value):
    cond_to_instr = {
        ("<", 0): "MOVGE",
        ("<", 1): "MOVLT",
        (">", 0): "MOVLE",
        (">", 1): "MOVGT",
    }
    instr = cond_to_instr.get((op, stack_length))

    if not instr:
        return False

    if not name_to_register.get(name):
        r = get_free_reg()
        name_to_register[name] = r

    instr_command = f"{instr} {name_to_register[name]} ,#{value}"
    add_instruction(instr_command)
    return True


def p_statement_assign(p):
    """statement : NAME "=" expression"""

    name = p[1]
    value = p[3]
    is_int = isinstance(value, int)

    if is_int and (value > 255 or value < -255):
        raise Exception(f"Integer too big")

    if len(stack) != 0:
        if handle_condition(stack.pop(), len(stack), name, value):
            return

    if is_int:
        if not name_to_register.get(name):
            r = get_free_reg()
            name_to_register[name] = r
        instr_command = f"MOV {name_to_register[name]} ,#{int(value)}"
        add_instruction(instr_command)
    else:
        name_to_register[name] = value


# handles math and comparison operators
def p_expression_math(p):
    """expression : expression '+' expression
    | expression '-' expression
    | expression '*' expression
    | expression '/' expression
    | expression '%' expression
    | expression '<' expression
    | expression '>' expression"""

    global current_register

    if p[2] not in ("<", ">"):
        registers_in_use_internally.append(get_free_reg())
        current_register = registers_in_use_internally.pop()

    if p[1] == None:
        p[1] = current_register
    if p[3] == None:
        p[3] = current_register
    if p[0] == None:
        p[0] = current_register

    if p[2] == "+":
        instr = "ADD " + p[0] + " ," + p[1] + " ," + p[3]
        add_instruction(instr)

    elif p[2] == "-":
        instr = "SUB " + p[0] + " ," + p[1] + " ," + p[3]
        add_instruction(instr)

    elif p[2] == "*":
        instr = "MUL " + p[0] + ", " + p[1] + ", " + p[3]
        add_instruction(instr)

    elif p[2] == "/":
        try:
            instr = "SDIV " + p[0] + ", " + p[1] + ", " + p[3]
            add_instruction(instr)
        except ZeroDivisionError:
            raise Exception(f"Division by zero error: {p}")

    elif p[2] == "%":
        try:
            instr1 = "SDIV " + p[0] + ", " + p[1] + ", " + p[3]
            instr2 = "MLS " + p[0] + ", " + p[0] + ", " + p[3] + ", " + p[1]
            add_instruction(instr1)
            add_instruction(instr2)
        except ZeroDivisionError:
            raise Exception(f"Division by zero error: {p}")

    elif p[2] in (">", "<"):
        instr1 = "CMP " + p[1] + ", " + p[3]
        stack.append(p[2])
        stack.append(p[2])
        add_instruction(instr1)


def p_expression_control_flow(p):
    """statement : expression '?' statement ':' statement
    | IF expression THEN statement ELSE statement"""


def p_expression_parenthesis(p):
    "expression : '(' expression ')'"
    p[0] = p[2]


def p_expression_number(p):
    "expression : NUMBER"
    p[0] = p[1]


def p_expression_name(p):
    "expression : NAME"
    name = p[1]

    if name in name_to_register:
        p[0] = name_to_register[name]
    else:
        raise NameError("Undefined name '%s'" % name)


def p_error(p):
    raise SyntaxError(f"Syntax error in input! (received: {p})")


# parser
yacc.yacc()


def main():
    # open the input file and parse it line by line
    input_file = "math.txt"
    with open(input_file, "r") as f:
        for line in f:
            yacc.parse(line)

    print(f" register mapping: {name_to_register}")
    print("".join(ASM))

    output_file = "output.s"
    with open(output_file, "w") as f:
        f.write("".join(ASM))


if __name__ == "__main__":
    main()
