; MACROS.ASM - Uso de macros para simplificar llamadas a DOS
.MODEL SMALL
.STACK 100h

Imprime MACRO cadena
    push ax
    push dx
    lea dx, cadena
    mov ah, 09h
    int 21h
    pop dx
    pop ax
ENDM

Termina MACRO codigo
    mov ah, 4Ch
    mov al, codigo
    int 21h
ENDM

.DATA
    linea1 DB 'Las macros se expanden al ensamblar.', 13, 10, '$'
    linea2 DB 'Cada uso copia su cuerpo en el codigo.', 13, 10, '$'

.CODE
inicio:
    mov ax, @data
    mov ds, ax
    Imprime linea1
    Imprime linea2
    Termina 0
END inicio
