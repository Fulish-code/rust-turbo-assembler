; SUMA.ASM - Lee dos digitos del teclado y muestra su suma (0..18)
.MODEL SMALL
.STACK 100h

.DATA
    pide1   DB 'Primer digito: $'
    pide2   DB 13, 10, 'Segundo digito: $'
    result  DB 13, 10, 'La suma es: $'

.CODE
inicio:
    mov ax, @data
    mov ds, ax

    lea dx, pide1
    mov ah, 09h
    int 21h
    mov ah, 01h            ; leer un caracter con eco
    int 21h
    sub al, '0'            ; ASCII -> numero
    mov bl, al

    lea dx, pide2
    mov ah, 09h
    int 21h
    mov ah, 01h
    int 21h
    sub al, '0'
    add bl, al             ; BL = suma

    lea dx, result
    mov ah, 09h
    int 21h

    mov al, bl
    xor ah, ah
    mov cl, 10
    div cl                 ; AL = decenas, AH = unidades
    mov bh, ah
    cmp al, 0
    je  unidades
    mov dl, al
    add dl, '0'
    mov ah, 02h            ; imprimir un caracter
    int 21h
unidades:
    mov dl, bh
    add dl, '0'
    mov ah, 02h
    int 21h

    mov ax, 4C00h
    int 21h
END inicio
