;**********************************************************
; BLINEAL3.ASM
;
;** Es el programa BLINEAL1.ASM del manual con la segunda versión del procedimiento blineal (direccionamiento
; registro indirecto).
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
;         AX = llave
;         BX = datos
;         CX = nDatos

;
; Regresa:
;
;          AX = pos si hay éxito, 0FFFFh en caso contrario.
;
; El pseudocódigo de este procedimiento es:
;
;    int blineal(int llave, int *pdatos, int ndatos)
;    {
;       pDatos = datos
;       p = pDatos + nDatos
;
;        while(pDatos < p)
;        {
;            if(llave == *pdatos) goto @@siesta
;            pDatos++
;        }
;
;        return -1
;
;    @@siesta:
;        return pDatos - datos
;    }
;**********************************************************
proc    blineal
        push    si                      ; Preserva SI

        mov     si, bx                  ; pDatos = SI = datos
        sal     cx, 1                   ; p = pDatos + 2*nDatos
        add     cx, bx

@@whi:  cmp     bx, cx                  ; while(pDatos < p)
        jae     @@endwhi                ; {
        cmp     ax, [bx]                ;    if(llave == *pDatos)
        je      @@siesta                ;       goto @@siesta
        inc     bx                      ;    pDatos++
        inc     bx                      ;    pDatos++
        jmp     @@whi                   ; }
@@endwhi:

        mov     ax, 0FFFFh              ; AX = 0FFFFh
        jmp     @@fin                   ; goto fin

@@siesta:
        mov     ax, bx                  ; AX = (pDatos-datos)/2
        sub     ax, si
        sar     ax, 1

@@fin:  pop     si                      ; Recupera SI
        ret
endp    blineal

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
