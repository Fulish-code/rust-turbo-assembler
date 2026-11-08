;**********************************************************
; SERIE3.ASM
;
; Este programa suma los números enteros de 1 hasta nfinal
; Versión que utiliza un procedimiento para obtener la
; suma.
;
; El pseudocódigo de este programa es:
;
;   AX = nfinal
;
;   serie()
;
;   suma = AX
;**********************************************************

;****** CÓDIGO DE INICIO **********************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA ****************************

        dataseg
codsal  db      0
nfinal  dw      ?
suma    dw      ?

;****** CÓDIGO DEL PROGRAMA *******************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        mov     ax, [nfinal]            ; AX = nfinal

        call    serie

        mov     [suma], ax              ; suma = AX
salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** PROCEDIMIENTOS ************************************

;**********************************************************
; SERIE
;
; Este procedimiento suma los números de 1 hasta nfinal
;
; Parámetro:
;
;        AX = nfinal
;
; Regresa:
;
;        AX = suma
;
; El pseudocódigo de este procedimiento es:
;
;   CX = nFinal
;   if(CX == 0) goto enddo
;   AX = 0
;
;   do
;   {
;      AX+= CX
;   }
;   while(--CX > 0)
;**********************************************************
proc    serie
        push    cx                      ; Preserva CX
        mov     cx, ax                  ; CX = nfinal

        jcxz    @@enddo                 ; if(CX == 0) goto enddo
        xor     ax, ax                  ; AX = 0

@@do:                                   ; do {
        add     ax, cx                  ;    AX += CX
        loop    @@do                    ; } while(--CX > 0)
@@enddo:

        pop     cx                      ; Recupera CX
        ret
endp    serie

;****** CÓDIGO DE TERMINACIÓN *****************************

        end     inicio
