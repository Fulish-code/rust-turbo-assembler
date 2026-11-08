;**********************************************************
; BLINEAL1.ASM
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
; Este procedimiento utiliza direccionamiento base indexado
; para encontrar la primera ocurrencia de llave dentro del
; arreglo datos.
;
; Parámetros:
;
;         AX = llave
;         BX = datos
;         CX = nDatos
;
; Regresa:
;
;         AX = pos si hay éxito, 0FFFFh en caso contrario
;
; El pseudocódigo para este procedimiento es:
;
;   int blineal(int llave, int *pdatos, int ndatos)
;   {
;      i = 0
;
;       while(i < nDatos)
;       {
;           if(llave == datos[i]) goto @@siesta
;           i++
;       }
;
;       return -1
;

;    @@siesta:
;        return I
;    }
;**********************************************************
proc    blineal
        push    si                      ; Preserva SI

        xor     si, si                  ; SI = 0
        sal     cx, 1                   ; CX = 2 * nDatos

@@whi:  cmp     si, cx                  ; while(SI < 2 * nDatos)
        jae     @@endwhi                ; {
        cmp     ax, [bx+si]             ;   if(llave ==
                                        ;       datos[SI])
        je      @@siesta                ;       goto @@siesta
        inc     si                      ;    SI++
        inc     si                      ;    SI++
        jmp     @@whi                   ; }
@@endwhi:
        mov     ax, 0FFFFh              ; AX = 0FFFFh
        jmp     @@fin                   ; goto fin

@@siesta:
        sar     si, 1                   ; SI /= 2
        mov     ax, si                  ; AX = CX

@@fin:  pop     si                      ; Recupera SI
        ret
endp    blineal

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
