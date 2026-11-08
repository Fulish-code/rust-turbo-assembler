;*************************************************************
; SERIE2.ASM
;
; Este programa suma los números enteros de 1 hasta nfinal.
; Esta versión utiliza la instrucción repetitiva loop. El
; pseudocódigo de este programa es:
;
;   AX = 0
;   CX = nfinal
;
;   if(CX == 0) goto enddo
;
;   do
;   {
;      AX += CX
;   }
;   while(--CX > 0)

; enddo:
;   suma = AX
;*************************************************************

;****** CÓDIGO DE INICIO *************************************

        ideal
        dosseg
        model   small
        stack   256

;****** VARIABLES DEL PROGRAMA *******************************

        dataseg
codsal  db      0
nfinal  dw      ?
suma    dw      ?

;****** CÓDIGO DEL PROGRAMA **********************************

        codeseg
inicio:
        mov     ax, @data               ; Inicializa el
        mov     ds, ax                  ; segmento de datos

        xor     ax, ax                  ; AX = 0
        mov     cx, [nfinal]            ; CX = nfinal

;** CORREGIDO: el manual dice jcxz endo; la etiqueta se llama enddo.
        jcxz    enddo                   ; if(CX == 0) goto enddo

do:                                     ; do
                                        ; {
        add     ax, cx                  ;    AX += CX
        loop    do                      ; }
                                        ; while(--CX > 0)
enddo:
        mov     [suma], ax              ; suma = AX

salir:
        mov     ah, 04Ch
        mov     al, [codsal]
        int     21h

;****** CÓDIGO DE TERMINACIÓN ********************************

        end     inicio
