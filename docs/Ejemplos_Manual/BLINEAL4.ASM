;**********************************************************
; BLINEAL4.ASM
;
;** Es el programa BLINEAL1.ASM del manual con la tercera versión del procedimiento blineal (recorre el
; arreglo de atrás hacia adelante).
;** El manual sólo muestra el procedimiento.
;
; Este programa busca la primera ocurrencia del dato dado
; por llave dentro del arreglo de palabras dado por datos.
; El programa almacena en la variable pos la posición de
; llave dentro del arreglo, -1 (0FFFFh) en caso contrario.
; El pseudocódigo de este programa es:
;
;   pos = blineal(llave, datos, ndatos)
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
llave   dw      ?
nDatos  dW      ?
pos     dW      ?
datos   dw      TAMMAX dup(?)

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [llave]             ; AX = llave
        mov     bx, offset datos        ; BX = datos
        mov     cx, [nDatos]            ; CX = nDatos

        call    blineal

        mov     [pos], ax               ; pos = AX

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; BLINEAL
;
; Este procedimiento utiliza el direccionamiento registro
; indirecto para encontrar la primera ocurrencia de llave
; dentro del arreglo datos.
;
; Parámetros:
;
;          AX = llave
;          BX = datos
;          CX = nDatos
;
; Regresa:
;
;          AX = pos si hay éxito, 0FFFFh en caso contrario
;
; El pseudocódigo de este procedimiento es:
;
;    int blineal(int llave, int *pdatos, int ndatos)
;    {
;       p = pDatos + nDatos
;
;        while(p > datos)
;        {
;            p--
;            if(llave == *p) goto @@siesta
;        }
;
;        return -1
;    @@siesta:
;        return p - datos
;    }
;**********************************************************
proc    blineal
        sal     cx, 1                   ; p = BX = datos
        add     cx, bx                  ; + 2*nDatos
        xchg    cx, bx                  ; CX = datos

@@whi:  cmp     bx, cx                  ; while(p > datos)
        jbe     @@endwhi                ; {
        dec     bx                      ;    p--
        dec     bx                      ;    p--
        cmp     ax, [bx]                ;    if(llave == *p)
        je      @@siesta                ;        goto @@siesta
        jmp     @@whi                   ; }

@@endwhi:

        mov     ax, 0FFFFh              ; AX = 0FFFFh
        ret

@@siesta:
        mov     ax, bx                  ; AX = (p - datos)/2
        sub     ax, cx
        sar     ax, 1
        ret
endp    blineal

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
