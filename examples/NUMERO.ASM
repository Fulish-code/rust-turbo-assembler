; NUMERO.ASM - Procedimientos: imprime un numero de 16 bits en decimal
.MODEL SMALL
.STACK 100h

.DATA
    texto DB 'El valor es: $'
    valor DW 31416

.CODE
inicio:
    mov ax, @data
    mov ds, ax

    lea dx, texto
    mov ah, 09h
    int 21h

    mov ax, valor
    call ImprimeDecimal

    mov ax, 4C00h
    int 21h

; Entrada: AX = numero sin signo. Usa la pila para invertir los digitos.
ImprimeDecimal PROC
    push ax
    push bx
    push cx
    push dx
    mov bx, 10
    xor cx, cx
divide:
    xor dx, dx
    div bx                 ; DX:AX / 10 -> AX cociente, DX residuo
    push dx
    inc cx
    test ax, ax
    jnz divide
saca:
    pop dx
    add dl, '0'
    mov ah, 02h
    int 21h
    loop saca
    pop dx
    pop cx
    pop bx
    pop ax
    ret
ImprimeDecimal ENDP

END inicio
