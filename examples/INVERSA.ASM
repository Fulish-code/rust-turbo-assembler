; INVERSA.ASM - Lee una cadena (INT 21h, AH=0Ah) y la imprime al reves
.MODEL SMALL
.STACK 100h

.DATA
    pide    DB 'Escribe algo: $'
    salto   DB 13, 10, 'Al reves: $'
    bufer   DB 60              ; tamano maximo
            DB ?               ; caracteres leidos
            DB 60 DUP(?)       ; texto

.CODE
inicio:
    mov ax, @data
    mov ds, ax

    lea dx, pide
    mov ah, 09h
    int 21h

    lea dx, bufer
    mov ah, 0Ah
    int 21h

    lea dx, salto
    mov ah, 09h
    int 21h

    xor cx, cx
    mov cl, [bufer+1]      ; cuantos caracteres se leyeron
    jcxz fin
    lea si, bufer+2
    add si, cx
    dec si                 ; SI apunta al ultimo caracter
imprime:
    mov dl, [si]
    mov ah, 02h
    int 21h
    dec si
    loop imprime
fin:
    mov ax, 4C00h
    int 21h
END inicio
