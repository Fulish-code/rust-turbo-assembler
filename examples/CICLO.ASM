; CICLO.ASM - Imprime los digitos del 0 al 9 usando LOOP
.MODEL SMALL
.STACK 100h

.CODE
inicio:
    mov cx, 10             ; numero de vueltas
    mov dl, '0'
otra:
    mov ah, 02h
    int 21h
    push dx
    mov dl, ' '
    int 21h
    pop dx
    inc dl
    loop otra

    mov ax, 4C00h
    int 21h
END inicio
