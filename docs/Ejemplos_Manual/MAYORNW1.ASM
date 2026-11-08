;**********************************************************
; MAYORNW1.ASM
;
; Este programa encuentra el mayor de un conjunto de datos
; de tipo palabra almacenados en un arreglo. El
; pseudocódigo de este programa es:
;
;     mayor = datos[0]
;     i = 0;
;
;     while(i < nDatos-1)
;     {
;        i++
;        if(mayor >= datos[i]) continue
;        mayor = datos[i]
;   }
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** DECLARACIÓN DE CONSTANTES SIMBÓLICAS **************

TAMMAX  equ     100

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
mayor   dw      ?
nDatos  dW      ?
datos   dw      TAMMAX dup(?)

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [datos]             ; mayor = datos[0]
        mov     bx, [nDatos]            ; BX = 2 *(nDatos-1)
        dec     bx
        sal     bx, 1
        xor     si, si                  ; SI = 0

;** CORREGIDO: el manual usa la etiqueta while; en TASM 4.1 WHILE es una
;** directiva (bloque de repetición), así que la etiqueta se llama whi.
whi:     cmp     si, bx                  ; while(SI < 2*nDatos)
        jae     endwhi                  ; {
        inc     si                      ;    SI++

        inc     si                      ;     SI++
        cmp     ax, [si+datos]          ;     if(mayor >= datos[SI])
        jae     whi                     ;        continue
        mov     ax, [si+datos]          ;     mayor = datos[SI]

        jmp     whi                     ; }
endwhi:
        mov     [mayor], ax

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
