;**********************************************************
; BLINEAL2.ASM
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
        mov     ds, ax                  ; segmento de datos y
        mov     es, ax                  ; el segmento extra

        mov     ax, [llave]             ; AX = llave
        mov     di, offset datos        ; DI = datos
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
; Este procedimiento utiliza la instrucción de cadenas
; scasw y el prefijo de repetición de instrucciones repne
; para encontrar la primera ocurrencia de llave dentro del
; arreglo datos.
;
; Parámetros:
;
;         AX = llave
;         DI = datos
;         CX = nDatos
;
; Regresa:
;
;         AX = pos si hay éxito, 0FFFFh en caso contrario.
;
; El pseudocódigo de este procedimiento es:
;
;   int blineal(int llave, int *datos, int ndatos)
;   {
;      n = nDatos
;
;      while(n-- > 0 && llave != *(datos++));
;
;      if(llave == *(datos-1) goto @@siesta
;
;      return -1

;
;    @@siesta:
;        return nDatos - (n+1)
;    }
;**********************************************************
proc    blineal
        push    cx                      ; Preserva CX

        cld                             ; Autoincrementa DI
        repne   scasw                   ; while(CX-- > 0 &&
                                        ;      llave != [DI++]);
        cmp     ax, [di-2]              ; if(llave==datos[DI-2])
        je      @@siesta                ;    goto @@siesta

        pop     cx                      ; Restaura CX
        mov     ax, 0FFFFh              ; AX = 0FFFFh
        ret

@@siesta:
        pop     ax                      ; AX = nDatos
        sub     ax, cx                  ; AX -= CX + 1
        dec     ax
        ret
endp    blineal

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
