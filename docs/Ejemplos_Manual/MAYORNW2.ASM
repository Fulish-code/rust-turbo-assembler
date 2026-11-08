;**********************************************************
; MAYORNW2.ASM
;
; Este programa encuentra el mayor de un conjunto de datos
; de tipo palabra almacenados en un arreglo. El
; pseudocódigo de este programa es:
;
;   pDatos = datos
;   p = datos + nDatos
;   mayor = *pDatos
;
;   while(pDatos < p-1)
;   {
;      pDatos++
;      if(mayor >= *pDatos continue
;      mayor = *pDatos
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

        mov     bx, offset datos        ; pDatos = datos
        mov     cx, [nDatos]            ; p = datos + (2*nDatos
        dec     cx                      ;    -1)
        sal     cx, 1
        add     cx, bx
        mov     ax, [bx]                ; mayor = *pDatos

;** CORREGIDO: el manual usa la etiqueta while; en TASM 4.1 WHILE es una
;** directiva (bloque de repetición), así que la etiqueta se llama whi.
whi:     cmp     bx, cx                  ; while(pDatos < p)
        jae     endwhi                  ; {
        inc     bx                      ;    pDatos++
        inc     bx                      ;    pDatos++

        cmp     ax, [bx]                ;     if(mayor >= *pDatos)
        jae     whi                     ;        continue
        mov     ax, [bx]                ;     mayor = *pDatos
        jmp     whi                     ; }
endwhi:
        mov     [mayor], ax

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
